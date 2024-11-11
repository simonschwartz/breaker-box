package circuitbreaker_test

import (
	"math/rand"
	"testing"
	"time"

	"circuitbreaker"

	"github.com/stretchr/testify/assert"
)

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

func FastForward(duration time.Duration, mockTime *MockTime) {
	mockTime.MockCurrentTime = mockTime.MockCurrentTime.Add(duration)
}

type MockTime struct {
	MockCurrentTime time.Time
}

func (m *MockTime) Now() time.Time {
	return m.MockCurrentTime
}

func TestCircuitBreaker(t *testing.T) {
	mockTime := &MockTime{
		MockCurrentTime: time.Now(),
	}

	cb := circuitbreaker.
		New().
		UNSAFESetTime(mockTime).
		SetEvalWindow(3, 3).
		SetErrorThreshold(10.0).
		Build()

	var rate float64
	var state circuitbreaker.State

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
		RecordErrors(s.errors, cb)
		RecordSuccesses(s.successes, cb)
		FastForward(61*time.Second, mockTime)
	}

	rate = cb.GetErrorRate()
	state = cb.GetState()

	assert.Equal(t, 4.12, rate)
	assert.Equal(t, circuitbreaker.Closed, state)

	// Second - simulate a spike in errors to Open the circuit
	RecordErrors(250, cb)
	RecordSuccesses(1, cb)
	FastForward(61*time.Second, mockTime)
	RecordErrors(1, cb)

	rate = cb.GetErrorRate()
	state = cb.GetState()

	assert.Equal(t, 0.0, rate)
	assert.Equal(t, circuitbreaker.Open, state)

	// Third - wait 1 minute for the circuit to move to HalfOpen
	FastForward(61*time.Second, mockTime)
	RecordSuccesses(1, cb)

	state = cb.GetState()
	assert.Equal(t, circuitbreaker.HalfOpen, state)

	// Fourth - oh no, an error, the circuit goes back to Open
	RecordErrors(1, cb)

	state = cb.GetState()
	assert.Equal(t, circuitbreaker.Open, state)

	// Fifth - wait 1 minute for the circuit to move to HalfOpen
	// Add 1- consecutive values so the circuit closes
	FastForward(61*time.Second, mockTime)
	// need to do this to get the state to update
	state = cb.GetState()
	assert.Equal(t, circuitbreaker.HalfOpen, state)

	RecordSuccesses(10, cb)

	state = cb.GetState()
	assert.Equal(t, circuitbreaker.Closed, state)
}

// Benchmarks - go test src/pkg/circuitbreaker/circuitbreaker_test.go -bench=BenchmarkCircuitBreakerRecord -benchtime=5s
func BenchmarkCircuitBreakerRecord(b *testing.B) {
	cb := circuitbreaker.New().Build()

	isErrors := make([]bool, b.N)
	for i := 0; i < b.N; i++ {
		// #nosec G404 -- Using math/rand is acceptable here as this is just for test purposes
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