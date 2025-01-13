package app

import (
	"time"
)

type Event int

const (
	Fail    Event = iota
	Success Event = iota
	None    Event = iota
)

type EventDisplayState struct {
	Event Event
	d     time.Duration
	timer *time.Timer
}

// When a user submits a Success or Failure event to the circuit breaker we want to display the event on the screen for a short period of time.
// NewEventDisplayState handles the setting and timed cleanup of the current visual event state
func NewEventDisplayState(d time.Duration) *EventDisplayState {
	return &EventDisplayState{
		Event: None,
		d:     d,
	}
}

func (e *EventDisplayState) resetTimer() {
	if e.timer == nil {
		e.timer = time.NewTimer(e.d)
	} else {
		// Reset() should only be called on a stopped or expired timer
		// We can safely call Stop() on a timer of any state to ensure we can safely reset
		e.timer.Stop()
		e.timer.Reset(e.d)
	}

	go func() {
		<-e.timer.C
		e.Event = None
		e.timer.Stop()
	}()
}

func (e *EventDisplayState) RecordEvent(event Event) {
	e.resetTimer()
	e.Event = event
}
