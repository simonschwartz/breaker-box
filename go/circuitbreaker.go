package circuitbreaker

import (
	"math"
	"sync"
	"time"

	"circuitbreaker/internal/ringbuffer"
	"circuitbreaker/internal/scheduler"
)

const (
	// The window, in minutes, of data to evaluate circuit breaker state
	DefaultEvalWindow = 10

	// Duration of data each node in the buffer stores that make up the overall evaluation window
	DefaultNodeDuration = time.Minute

	// The minimum amount of events required to evaluate circuit breaker state
	DefaultMinEvalSize = 100

	// Error rate threshold required to trip the circuit breaker
	DefaultErrorThreshold = 10.0

	// Default number of consecutive success events required to move HalfOpen circuit to Closed
	DefaultTrialSuccessesRequired = 20

	// Duration to wait to move Open circuit to HalfOpen
	DefaultRetryTimeout = time.Minute
)

type State int

const (
	Open     State = iota
	HalfOpen State = iota
	Closed   State = iota
)

type Result int

const (
	Success Result = iota
	Failure Result = iota
)

type BufferNode struct {
	Index        int
	Expires      time.Time
	FailureCount int
	SuccessCount int
}

type Status struct {
	State          State
	ErrorRate      float64
	TrialSuccesses int
	BufferNodes    []*BufferNode
	ActiveNode     *BufferNode
	Config
}

func (n *BufferNode) Init(index int) {
	n.Index = index
	n.Expires = time.Time{}
	n.FailureCount = 0
	n.SuccessCount = 0
}

func (n *BufferNode) Reset(expires time.Time) {
	n.Expires = expires
	n.FailureCount = 0
	n.SuccessCount = 0
}

type CircuitBreaker struct {
	mu        sync.RWMutex
	state     State
	buffer    *ringbuffer.RingBuffer[BufferNode]
	errorRate float64
	time      ITime
	// time based scheduler for advancing the ring buffer at regular intervals
	cursorScheduler *scheduler.Scheduler
	// schedules the timing for transitioning from Open to HalfOpen state
	retryScheduler *scheduler.Scheduler
	// how many consecutive successes have occurred while circuit is HalfOpen
	trialSuccesses int
	config         Config
}
type Config struct {
	// duration of data each node in the buffer stores
	NodeDuration time.Duration
	// minimum number of events required in the buffer to evaluate the error rate
	MinEvalSize int
	// percentage of errors that will cause the circuit to Open
	ErrorThreshold float64
	// duration the circuit breaker remains in the Open state before transitioning to HalfOpen.
	RetryTimeout time.Duration
	// how many successive successes required to close a half open circuit
	TrialSuccessesRequired int
}

type Builder struct {
	cb *CircuitBreaker
}

type ITime interface {
	Now() time.Time
	NewTicker(d time.Duration) *time.Ticker
}

type Time struct{}

func (Time) Now() time.Time {
	return time.Now()
}

func (Time) NewTicker(d time.Duration) *time.Ticker {
	return time.NewTicker(d)
}

// UNSAFE - only intended for use by internal testing tools
// SetTime allows setting a custom time provider for the circuit breaker.
// This is particularly(only?) useful for unit testing, where you may need to control or simulate time progression.
func (b *Builder) UNSAFESetTime(time ITime) *Builder {
	b.cb.time = time
	return b
}

// SetEvalWindow configures the evaluation window for the circuit breaker.
// The evaluation window determines the duration and granularity of data considered
// when assessing the circuit breaker state.
//
// Parameters:
//   - minutes: The total duration of the evaluation window in minutes.
//     Defaults to 10 minutes if not specified.
//   - nodes: The number of nodes the evaluation window is divided into.
//     This allows for more granular data collection and analysis.
//
// The circuit breaker requires data for the full evaluation window before making state decisions.
//
// Example: SetEvalWindow(10, 5) creates an evaluation window of 10 minutes
// divided into 5 nodes of 2 minutes each.
func (b *Builder) SetEvalWindow(minutes int, nodes int) *Builder {
	if minutes <= 0 {
		minutes = DefaultEvalWindow
	}

	if nodes <= 0 {
		b.cb.config.NodeDuration = DefaultNodeDuration
		b.cb.buffer = ringbuffer.New[BufferNode](minutes + 1)
		return b
	}

	b.cb.config.NodeDuration = time.Duration(float64(minutes) / float64(nodes) * float64(time.Minute))
	b.cb.buffer = ringbuffer.New[BufferNode](nodes + 1)

	return b
}

// SetMinEvalSize sets the minimum number of events required within the evaluation window
// to assess the error rate and determine the circuit breaker state.
//
// If not set, the defaults is 100 events.
func (b *Builder) SetMinEvalSize(size int) *Builder {
	b.cb.config.MinEvalSize = size
	return b
}

// SetErrorThreshold configures the error rate threshold for the circuit breaker.
// The threshold is a percentage (0-100) of failed requests relative to total requests.
// When the error rate exceeds this threshold, the circuit breaker will open.
//
// If not set, the default threshold is 10.0 (10%)
func (b *Builder) SetErrorThreshold(threshold float64) *Builder {
	b.cb.config.ErrorThreshold = threshold
	return b
}

// SetRetryTimeout configures the duration the circuit breaker remains in the Open state before
// transitioning to Half-Open. This timeout represents a "cooling off" period, allowing the underlying
// system time to recover before the circuit breaker cautiously allows traffic through again in the Half-Open state.
//
// Note: Setting a very short timeout might lead to rapid oscillation between Open and Half-Open states
// if the underlying system hasn't fully recovered. Conversely, a very long timeout might
// unnecessarily delay recovery if the system issues are resolved quickly.
//
// If not set, the default in time.Minute (1 minute)
func (b *Builder) SetRetryTimeout(duration time.Duration) *Builder {
	b.cb.config.RetryTimeout = duration
	return b
}

// SetTrialSuccessesRequired configures the number of consecutive successful requests needed
// while in the Half-Open state before the circuit breaker transitions back to the Closed state.
//
// This acts as a confidence threshold - requiring multiple successful requests helps ensure
// the underlying system has truly recovered before fully restoring traffic.
//
// Note: Setting this value too low might result in premature recovery if the system
// is still unstable. Setting it too high might unnecessarily delay recovery.
//
// If not set, the default is 20 successful requests
func (b *Builder) SetTrialSuccessesRequired(number int) *Builder {
	b.cb.config.TrialSuccessesRequired = number
	return b
}

func (b *Builder) Build() *CircuitBreaker {
	b.cb.cursorScheduler = scheduler.New(b.cb.time, b.cb.config.NodeDuration, b.cb.moveCursor)
	b.cb.retryScheduler = scheduler.New(b.cb.time, b.cb.config.RetryTimeout, func() {
		b.cb.mu.Lock()
		defer b.cb.mu.Unlock()

		b.cb.state = HalfOpen
		b.cb.retryScheduler.Stop()
	})

	b.cb.cursorScheduler.Start()
	b.cb.initBuffer()
	b.cb.buffer.Cursor().Reset(b.cb.time.Now().Add(b.cb.config.NodeDuration))

	return b.cb
}

func New() *Builder {
	return &Builder{
		cb: &CircuitBreaker{
			state:     Closed,
			buffer:    ringbuffer.New[BufferNode](DefaultEvalWindow + 1),
			errorRate: 0.00,
			time:      &Time{},
			config: Config{
				NodeDuration:           DefaultNodeDuration,
				MinEvalSize:            DefaultMinEvalSize,
				ErrorThreshold:         DefaultErrorThreshold,
				TrialSuccessesRequired: DefaultTrialSuccessesRequired,
				RetryTimeout:           DefaultRetryTimeout,
			},
		},
	}
}

func (cb *CircuitBreaker) moveCursor() {
	cb.mu.Lock()
	defer cb.mu.Unlock()

	cb.buffer.Next()
	cb.buffer.Cursor().Reset(cb.time.Now().Add(cb.config.NodeDuration))
	cb.errorRate = cb.calculateErrorRate(cb.config.MinEvalSize)

	if cb.state == Closed && cb.errorRate > cb.config.ErrorThreshold {
		cb.state = Open
		cb.clearBuffer()
		cb.errorRate = 0.00
		cb.cursorScheduler.Stop()
		cb.retryScheduler.Start()
	}
}

// When starting the circuit breaker we must initialise each node in the buffer with default values
func (cb *CircuitBreaker) initBuffer() {
	index := 0

	cb.buffer.DoFromHead(func(node *BufferNode) {
		node.Index = index
		node.Expires = time.Time{}
		node.FailureCount = 0
		node.SuccessCount = 0

		index++
	})
}

func (cb *CircuitBreaker) clearBuffer() {
	cb.buffer.Do(func(node *BufferNode) {
		node.Expires = time.Time{}
		node.FailureCount = 0
		node.SuccessCount = 0
	})
}

func (cb *CircuitBreaker) GetState() State {
	cb.mu.RLock()
	defer cb.mu.RUnlock()

	return cb.state
}

func (cb *CircuitBreaker) Record(result Result) {
	cb.mu.Lock()
	defer cb.mu.Unlock()

	if cb.state == Open {
		return
	}

	// If the circuit is HalfOpen, allow a small sample or trial traffic through
	// If 20 consecutive successes occur, assume the service is OK and set the circuit to Closed
	if cb.state == HalfOpen && result == Success {
		cb.trialSuccesses++
		if cb.trialSuccesses >= cb.config.TrialSuccessesRequired {
			cb.state = Closed
			cb.trialSuccesses = 0
			cb.cursorScheduler.Start()
			cb.buffer.Cursor().Reset(cb.time.Now().Add(cb.config.NodeDuration))
		}
		return
	}

	// If the circuit is HalfOpen, allow a small sample of trial traffic through
	// If an error occurs during the trial, assume the service is still unavailable and set the circuit to Open
	if cb.state == HalfOpen && result == Failure {
		cb.state = Open
		cb.trialSuccesses = 0
		cb.retryScheduler.Start()
		return
	}

	if result == Failure {
		cb.buffer.Cursor().FailureCount++
	} else {
		cb.buffer.Cursor().SuccessCount++
	}
}

func (cb *CircuitBreaker) calculateErrorRate(minEvalSize int) float64 {
	failures := 0
	total := 0
	skippedActiveNode := false

	cb.buffer.Do(func(node *BufferNode) {
		if !skippedActiveNode {
			skippedActiveNode = true
			return
		}
		failures += node.FailureCount
		total += +node.FailureCount + node.SuccessCount
	})

	if total < minEvalSize || total == 0 {
		return 0
	}

	errorRate := (float64(failures) / float64(total)) * 100
	return math.Round(errorRate*100) / 100
}

// Inspect the current state and config of the circuit breaker for reporting or debugging
func (cb *CircuitBreaker) Inspect() *Status {
	cb.mu.RLock()
	defer cb.mu.RUnlock()

	var bufferNodes []*BufferNode

	cb.buffer.DoFromHead(func(node *BufferNode) {
		bufferNodes = append(bufferNodes, node)
	})

	return &Status{
		// 0 = Open, 1 = HalfOpen, 2 = Closed
		State:     cb.state,
		ErrorRate: cb.errorRate,
		// Current state of data in each buffer node
		BufferNodes: bufferNodes,
		ActiveNode:  cb.buffer.Cursor(),
		// Number of consecutive successful requests. Will only have a value when the circuit breaker is in HalfOpen state
		TrialSuccesses: cb.trialSuccesses,
		Config: Config{
			NodeDuration:           cb.config.NodeDuration,
			MinEvalSize:            cb.config.MinEvalSize,
			ErrorThreshold:         cb.config.ErrorThreshold,
			TrialSuccessesRequired: cb.config.TrialSuccessesRequired,
			RetryTimeout:           cb.config.RetryTimeout,
		},
	}
}
