package circuitbreaker

import (
	"sync"
	"time"
)

const (
	// The window, in minutes, of data to evaluate circuit breaker state
	DefaultEvalWindow = 10

	// Duration of data each node/span in the buffer stores that make up the overall evaluation window
	DefaultSpanDuration = time.Minute

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

type CircuitBreaker struct {
	mu        sync.RWMutex
	state     State
	buffer    *RingBuffer
	errorRate float64
	time      ITime

	// time based scheduler for advancing the ring buffer at regular intervals
	cursorScheduler *Scheduler

	// schedules the timing for transitioning from Open to HalfOpen state
	retryScheduler *Scheduler

	// duration of data each node/span in the buffer stores
	spanDuration time.Duration

	// Minimum number of events required in the buffer to evaluate the error rate
	minEvalSize int

	// percentage of errors that will cause the circuit to Open
	errorThreshold float64

	// duration the circuit breaker remains in the Open state before transitioning to HalfOpen.
	retryTimeout time.Duration

	// how many successive successes required to close a half open circuit
	trialSuccessesRequired int

	// how many successive successes have occurred while circuit is HalfOpen
	trialSuccesses int
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
//   - spans: The number of time spans the evaluation window is divided into.
//     This allows for more granular data collection and analysis.
//
// The circuit breaker requires data for the full evaluation window before making state decisions.
//
// Example: SetEvalWindow(10, 5) creates an evaluation window of 10 minutes
// divided into 5 spans of 2 minutes each.
func (b *Builder) SetEvalWindow(minutes int, spans int) *Builder {
	if minutes <= 0 {
		minutes = DefaultEvalWindow
	}

	if spans <= 0 {
		b.cb.spanDuration = DefaultSpanDuration
		b.cb.buffer = NewRingBuffer(minutes + 1)
		return b
	}

	b.cb.spanDuration = time.Duration(float64(minutes) / float64(spans) * float64(time.Minute))
	b.cb.buffer = NewRingBuffer(spans + 1)

	return b
}

// SetMinEvalSize sets the minimum number of events required within the evaluation window
// to assess the error rate and determine the circuit breaker state.
//
// If not set, the defaults is 100 events.
func (b *Builder) SetMinEvalSize(size int) *Builder {
	b.cb.minEvalSize = size
	return b
}

// SetErrorThreshold configures the error rate threshold for the circuit breaker.
// The threshold is a percentage (0-100) of failed requests relative to total requests.
// When the error rate exceeds this threshold, the circuit breaker will open.
//
// If not set, the default threshold is 10.0 (10%)
func (b *Builder) SetErrorThreshold(threshold float64) *Builder {
	b.cb.errorThreshold = threshold
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
	b.cb.retryTimeout = duration
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
	b.cb.trialSuccessesRequired = number
	return b
}

func (b *Builder) Build() *CircuitBreaker {
	b.cb.cursorScheduler = NewScheduler(b.cb.time, b.cb.spanDuration, b.cb.moveCursor)
	b.cb.retryScheduler = NewScheduler(b.cb.time, b.cb.retryTimeout, func() {
		b.cb.mu.Lock()
		defer b.cb.mu.Unlock()

		b.cb.state = HalfOpen
		b.cb.retryScheduler.Stop()
	})

	b.cb.cursorScheduler.Start()
	b.cb.buffer.Cursor().Reset(b.cb.time.Now().Add(b.cb.spanDuration))

	return b.cb
}

func New() *Builder {
	return &Builder{
		cb: &CircuitBreaker{
			state:                  Closed,
			buffer:                 NewRingBuffer(DefaultEvalWindow + 1),
			errorRate:              0.00,
			spanDuration:           DefaultSpanDuration,
			time:                   &Time{},
			minEvalSize:            DefaultMinEvalSize,
			errorThreshold:         DefaultErrorThreshold,
			trialSuccessesRequired: DefaultTrialSuccessesRequired,
			retryTimeout:           DefaultRetryTimeout,
		},
	}
}

func (cb *CircuitBreaker) moveCursor() {
	cb.mu.Lock()
	defer cb.mu.Unlock()

	cb.buffer.Next()
	cb.buffer.Cursor().Reset(cb.time.Now().Add(cb.spanDuration))
	cb.errorRate = cb.buffer.GetErrorRate(cb.minEvalSize)

	if cb.state == Closed && cb.errorRate > cb.errorThreshold {
		cb.state = Open
		cb.buffer.ClearBuffer()
		cb.errorRate = 0.00
		cb.cursorScheduler.Stop()
		cb.retryScheduler.Start()
	}
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
		if cb.trialSuccesses >= cb.trialSuccessesRequired {
			cb.state = Closed
			cb.trialSuccesses = 0
			cb.cursorScheduler.Start()
			cb.buffer.Cursor().Reset(cb.time.Now().Add(cb.spanDuration))
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

func (cb *CircuitBreaker) GetErrorRate() float64 {
	cb.mu.RLock()
	defer cb.mu.RUnlock()

	return cb.errorRate
}
