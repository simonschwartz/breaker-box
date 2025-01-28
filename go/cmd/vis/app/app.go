package app

import (
	"fmt"
	"os"
	"time"

	"circuitbreaker"

	tea "github.com/charmbracelet/bubbletea"
)

const (
	refreshInterval = 100 * time.Millisecond
)

type model struct {
	circuitBreaker *circuitbreaker.CircuitBreaker
	es             *EventDisplayState
	ui             *UI
}

func initialModel(config *ProgramConfig) model {
	cb := circuitbreaker.New().
		SetEvalWindow(config.WindowDuration, config.Spans-1).
		SetMinEvalSize(config.MinEvalSize).
		SetErrorThreshold(config.ErrorThreshold).
		SetRetryTimeout(time.Duration(config.RetryTimeout) * time.Second).
		SetTrialSuccessesRequired(config.TrialSuccessesRequired).
		Build()

	es := NewEventDisplayState(300 * time.Millisecond)

	ui := NewUI(cb, es)

	return model{
		circuitBreaker: cb,
		es:             es,
		ui:             ui,
	}
}

func tickEvery(duration time.Duration) tea.Cmd {
	return tea.Every(duration, func(t time.Time) tea.Msg {
		return t
	})
}

func (m model) Init() tea.Cmd {
	return tickEvery(refreshInterval)
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			return m, tea.Quit
		case "f":
			m.circuitBreaker.Record(circuitbreaker.Failure)
			m.es.RecordEvent(Fail)
		case "s":
			m.circuitBreaker.Record(circuitbreaker.Success)
			m.es.RecordEvent(Success)
		}
	case time.Time:
		return m, tickEvery(refreshInterval)
	}
	return m, nil
}

func (m model) View() string {
	ui := m.ui.Render()
	return ui
}

func runNewProgram(config *ProgramConfig) {
	p := tea.NewProgram(initialModel(config))

	if _, err := p.Run(); err != nil {
		fmt.Printf("Failed to run visualiser testing tool: %v", err)
		os.Exit(1)
	}
}
