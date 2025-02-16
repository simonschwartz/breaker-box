package circuitbreaker_test

import (
	"math/rand"
	"sync"
	"testing"
	"time"

	"circuitbreaker"
)

type MockTicker struct {
	C        chan time.Time
	duration time.Duration
	lastTick time.Time
	mockTime *MockTime
}

type MockTime struct {
	MockCurrentTime time.Time
	tickers         []*MockTicker
	mu              sync.Mutex
}

func NewMockTime(initialTime time.Time) *MockTime {
	return &MockTime{
		MockCurrentTime: initialTime,
		tickers:         make([]*MockTicker, 0),
	}
}

func (m *MockTime) Now() time.Time {
	return m.MockCurrentTime
}

func (m *MockTime) NewTicker(d time.Duration) *time.Ticker {
	m.mu.Lock()
	defer m.mu.Unlock()

	mockTicker := &MockTicker{
		C:        make(chan time.Time, 10),
		duration: d,
		lastTick: m.MockCurrentTime,
		mockTime: m,
	}

	m.tickers = append(m.tickers, mockTicker)

	return &time.Ticker{
		C: mockTicker.C,
	}
}

func (m *MockTime) removeTicker(ticker *MockTicker) {
	m.mu.Lock()
	defer m.mu.Unlock()

	for i, t := range m.tickers {
		if t == ticker {
			m.tickers = append(m.tickers[:i], m.tickers[i+1:]...)
			return
		}
	}
}

func (m *MockTime) FastForward(duration time.Duration) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.MockCurrentTime = m.MockCurrentTime.Add(duration)

	for _, ticker := range m.tickers {
		// Check if, given the duration, the ticker should be fired
		next := ticker.lastTick.Add(ticker.duration)
		if next.Before(m.MockCurrentTime) || next.Equal(m.MockCurrentTime) {
			ticker.C <- ticker.lastTick.Add(ticker.duration)
			ticker.lastTick = ticker.lastTick.Add(ticker.duration)
		}
	}

	// Internally the circuit breaker sets up functions in goroutines that are triggered by ticks.
	// There is a very small delay between a tick occurs and the callback function runs because we need to wait for Go to schedule the goroutine after the tick occurs
	// To get around this we add a small delay(not ideal) to give the Go runtime a chance to run the goroutine
	time.Sleep(1 * time.Millisecond)
}

func (t *MockTicker) Stop() {
	t.mockTime.removeTicker(t)
	close(t.C)
}

func assert[T comparable](t *testing.T, actual T, expected T) {
	t.Helper()
	if actual != expected {
		t.Errorf("got %v, want %v", actual, expected)
	}
}

func RecordErrors(num int, cb *circuitbreaker.CircuitBreaker) {
	for i := 0; i < num; i++ {
		cb.Record(circuitbreaker.Failure)
	}
}

func RecordSuccesses(num int, cb *circuitbreaker.CircuitBreaker) {
	for i := 0; i < num; i++ {
		cb.Record(circuitbreaker.Success)
	}
}

func TestCircuitBreaker(t *testing.T) {
	mockTime := NewMockTime(time.Now())

	cb := circuitbreaker.
		New().
		UNSAFESetTime(mockTime).
		SetEvalWindow(3, 3).
		SetErrorThreshold(10.0).
		Build()

	// First - fill the buffer with events
	fillBuffer := []struct {
		errors    int
		successes int
	}{
		{
			errors:    0,
			successes: 200,
		},
		{
			errors:    22,
			successes: 143,
		},
		{
			errors:    1,
			successes: 100,
		},
		{
			errors:    0,
			successes: 292,
		},
		{
			errors:    5,
			successes: 192,
		},
	}

	for _, s := range fillBuffer {
		mockTime.FastForward(61 * time.Second)
		RecordErrors(s.errors, cb)
		RecordSuccesses(s.successes, cb)
	}

	status := cb.Inspect()

	assert(t, status.ErrorRate, 4.12)
	assert(t, circuitbreaker.Closed, status.State)

	// Second - simulate a spike in errors to Open the circuit
	RecordErrors(250, cb)
	RecordSuccesses(1, cb)
	mockTime.FastForward(61 * time.Second)
	RecordErrors(1, cb)

	status = cb.Inspect()

	assert(t, circuitbreaker.Open, status.State)
	assert(t, status.ErrorRate, 0.0)

	// Third - wait 1 minute for the circuit to move to HalfOpen
	mockTime.FastForward(61 * time.Second)
	RecordSuccesses(1, cb)

	status = cb.Inspect()
	assert(t, circuitbreaker.HalfOpen, status.State)

	// Fourth - oh no, an error, the circuit goes back to Open
	RecordErrors(1, cb)

	status = cb.Inspect()
	assert(t, circuitbreaker.Open, status.State)

	// Fifth - wait 1 minute for the circuit to move to HalfOpen
	mockTime.FastForward(61 * time.Second)

	status = cb.Inspect()

	assert(t, circuitbreaker.HalfOpen, status.State)

	RecordSuccesses(20, cb)

	status = cb.Inspect()
	assert(t, circuitbreaker.Closed, status.State)
}

// Record() is the most frequently used method in the Circuit Breaker
func BenchmarkCircuitBreakerRecord(b *testing.B) {
	cb := circuitbreaker.New().Build()

	isErrors := make([]bool, b.N)
	for i := 0; i < b.N; i++ {
		isErrors[i] = rand.Float32() < 0.1
	}

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		if isErrors[i] {
			cb.Record(circuitbreaker.Failure)
		} else {
			cb.Record(circuitbreaker.Success)
		}
	}
}

// GetState() may be frequently used by consumers to determine if they will defer sending traffic to a unavailable service
func BenchmarkCircuitGetState(b *testing.B) {
	cb := circuitbreaker.New().Build()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		cb.GetState()
	}
}

// This is the recommended way to integrate the circuit breaker
func BenchmarkCircuitBreaker(b *testing.B) {
	cb := circuitbreaker.
		New().
		SetEvalWindow(1, 6).
		SetErrorThreshold(20.0).
		Build()

	isErrors := make([]bool, b.N)
	for i := 0; i < b.N; i++ {
		isErrors[i] = rand.Float32() < 0.1
	}

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		state := cb.GetState()

		if state != circuitbreaker.Open {
			if isErrors[i] {
				cb.Record(circuitbreaker.Failure)
			} else {
				cb.Record(circuitbreaker.Success)
			}
		}
	}
}
