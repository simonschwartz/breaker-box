package circuitbreaker

import (
	"sync"
	"time"
)

type TimeTicker interface {
	NewTicker(d time.Duration) *time.Ticker
}

type Scheduler struct {
	time      TimeTicker
	stop      chan any
	callback  func()
	duration  time.Duration
	isRunning bool
	mu        sync.Mutex
}

// Schedule a callback function to be executed at specified intervals
func NewScheduler(time TimeTicker, duration time.Duration, callback func()) *Scheduler {
	return &Scheduler{
		duration: duration,
		callback: callback,
		time:     time,
	}
}

func (s *Scheduler) Start() {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.isRunning {
		return
	}

	timer := s.time.NewTicker(s.duration)
	s.stop = make(chan any, 1)
	s.isRunning = true

	go func() {
		for {
			select {
			case <-timer.C:
				s.callback()
			case <-s.stop:
				s.isRunning = false
				timer.Stop()
				return
			}
		}
	}()
}

func (s *Scheduler) Stop() {
	s.mu.Lock()
	defer s.mu.Unlock()

	if !s.isRunning {
		return
	}

	close(s.stop)
}
