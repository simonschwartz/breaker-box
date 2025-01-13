// The ASCII UI for the CLI app is rendered using a scanline approach, similar to an old CRT display.
// The premise is we describe the UI as rows in an array, then join them when we are ready to render
//
// Example:
//
// 	ui := []string{
// 		"[node0] ─▶ [node1] ─▶ [node2]",
// 		"   ▲                     │   ",
// 		"   │                     ▼   ",
// 		"   └──────────────────[node3]",
// 	}
// 	strings.Join(ui, "\n")

package app

import (
	"circuitbreaker"
	"fmt"
	"math"
	"strings"
	"time"
)

//gofmt:off
// The BufferLayout describes the order and layout of buffers.
//
// For example a BufferLayout of 4 nodes should be represented as:
// BufferLayout{
// 	Top:    []int{0, 1, 2},
// 	Middle: [2]MiddleBuffer{},
// 	Bottom: []int{3},
// }
//
// Which should be rendered as:
// [node0] ─▶ [node1] ─▶ [node2]
//    ▲                     │
//    │                     ▼
//    └──────────────────[node3]
//
// When the number of buffers exceeds 6 we must connect the top to bottom
// with columns of buffers described using the MiddleBuffer.
//
// For example a BufferLayout of 9 nodes should be represented as:
// BufferLayout{
// 	Top:    []int{0, 1, 2},
// 	Middle: [2]MiddleBuffer{
// 		[]int{-1, 8}
// 		[]int{3, 4}
// 	},
// 	Bottom: []int{7, 6, 5},
// }
//
// Note: -1 represents an empty slot.
//
// Which should be rendered as:
// [node0] ─▶ [node1] ─▶ [node2]
//    ▲                     │
//    │                     ▼
//    │                  [node3]
//    │                     │
//    │                     ▼
// [node8]               [node4]
//    ▲                     │
//    │                     ▼
// [node7] ─▶ [node6] ─▶ [node5]
//gofmt:on

type BufferLayout struct {
	Top    []int
	Middle [2]MiddleBuffer
	Bottom []int
}

type MiddleBuffer = []int

type UI struct {
	cb         *circuitbreaker.CircuitBreaker
	es         *EventDisplayState
	retryStart time.Time
}

func NewUI(cb *circuitbreaker.CircuitBreaker, es *EventDisplayState) *UI {
	return &UI{
		cb: cb,
		es: es,
	}
}

func (ui *UI) generateBufferLayout() BufferLayout {
	buffersLength := ui.cb.UNSAFEGetBufferLength()

	switch buffersLength {
	case 2:
		return BufferLayout{
			Top:    []int{0, 1, -1},
			Middle: [2]MiddleBuffer{},
			Bottom: []int{},
		}
	case 3:
		return BufferLayout{
			Top:    []int{0, 1, 2},
			Middle: [2]MiddleBuffer{},
			Bottom: []int{},
		}
	case 4:
		return BufferLayout{
			Top:    []int{0, 1, 2},
			Middle: [2]MiddleBuffer{},
			Bottom: []int{-1, -1, 3},
		}
	case 5:
		return BufferLayout{
			Top:    []int{0, 1, 2},
			Middle: [2]MiddleBuffer{},
			Bottom: []int{-1, 4, 3},
		}
	case 6:
		return BufferLayout{
			Top:    []int{0, 1, 2},
			Middle: [2]MiddleBuffer{},
			Bottom: []int{5, 4, 3},
		}
	default:
		// this path handles cases where there are more than 6 buffers, for these cases we need to have middle buffers to connect the top and bottom rows

		// We can assume that the top row will be full
		top := []int{0, 1, 2}

		// given we will always fill top row(3 items) and bottom row(3 items)
		// we can calculate how many items will be between the top and bottom rows
		middleBuffers := buffersLength - 6

		// we want to fill the right column then the left column from bottom up eg:
		//    ▲                     │
		//    │                     ▼
		//    │                  [node3]
		//    │                     │
		//    │                     ▼
		// [node8]               [node4]
		//    ▲                     │
		//    │                     ▼
		rightColLength := int(math.Ceil(float64(middleBuffers) / 2))
		leftColBuffers := int(math.Floor(float64(middleBuffers) / 2))

		// Given the number of items in the left column, we can fill the 3 items in the bottom row
		// counting back from the total number of buffers
		bottom := make([]int, 3)
		for i := 0; i < 3; i++ {
			bottom[i] = buffersLength - leftColBuffers - 1 - i
		}

		// Fill the right column
		rightCol := make([]int, rightColLength)
		for i := 0; i < rightColLength; i++ {
			rightCol[i] = i + 3
		}

		// Fill the left column - we can assume the left column will either be the same length, or 1 less that the right column
		leftCol := make([]int, rightColLength)
		diff := rightColLength - leftColBuffers
		if diff > 0 {
			leftCol[0] = -1
		}

		for i := 0; i < leftColBuffers; i++ {
			leftCol[diff+i] = 6 + rightColLength + leftColBuffers - i - 1
		}

		middle := [2]MiddleBuffer{leftCol, rightCol}

		return BufferLayout{
			Top:    top,
			Middle: middle,
			Bottom: bottom,
		}
	}
}

func (ui *UI) renderBufferBoxTop(index int) string {
	isActive := ui.cb.UNSAFEGetCursorIndex() == index
	if isActive {
		return "┏━━━━━━━━━━━━━━━━━┓"
	}
	return "┌─────────────────┐"
}

func (ui *UI) renderBufferBoxMiddle(index int) string {
	isActive := ui.cb.UNSAFEGetCursorIndex() == index
	cursor := ui.cb.UNSAFEGetCursorByIndex(index)
	if isActive {
		return fmt.Sprintf("┃ B%-2d \x1b[42m %03d \x1b[0m \x1b[41m %03d \x1b[0m ┃", index, cursor.SuccessCount, cursor.FailureCount)
	}
	return fmt.Sprintf("│ B%-2d \x1b[42m %03d \x1b[0m \x1b[41m %03d \x1b[0m │", index, cursor.SuccessCount, cursor.FailureCount)

}

func (ui *UI) renderBufferBoxBottom(index int) string {
	isActive := ui.cb.UNSAFEGetCursorIndex() == index
	if isActive {
		return "┗━━━━━━━━━━━━━━━━━┛"
	}
	return "└─────────────────┘"
}

// Visually renders the ring of buffers
func (ui *UI) renderBufferNodes() string {
	bl := ui.generateBufferLayout()
	// we render the UI using scanlines
	top := make([]string, 3)

	for i, b := range bl.Top {
		if b == -1 && i == 1 {
			top[1] += "────────────────────────────────┐"
			top[2] += "                                │"
			break
		}

		if b == -1 && i == 2 {
			top[1] += "───────────┐"
			top[2] += "           │"
			break
		}

		if i > 0 {
			top[0] += "  "
			top[1] += "─▶"
			top[2] += "  "
		}

		top[0] += ui.renderBufferBoxTop(b)
		top[1] += ui.renderBufferBoxMiddle(b)
		top[2] += ui.renderBufferBoxBottom(b)
	}

	var middle []string

	// if no middle buffer nodes, just connect top and bottom with arrows
	if len(bl.Middle[1]) == 0 {
		if len(bl.Bottom) > 0 {
			middle = append(middle, "         ▲                                         │")
			middle = append(middle, "         │                                         ▼")
		} else {
			middle = append(middle, "         ▲                                         │")
			middle = append(middle, "         └─────────────────────────────────────────┘")
		}
	} else {
		// middles exist
		middle = append(middle, "         ▲                                         │")
		middle = append(middle, "         │                                         ▼")

		// loop second column, as it will always be longer
		for i := range bl.Middle[1] {
			var scanlineTop string
			var scanlineMiddle string
			var scanlineBottom string

			leftBuffer := bl.Middle[0][i]
			rightBuffer := bl.Middle[1][i]

			if leftBuffer == -1 {
				// render line as no buffer exists
				scanlineTop += "         │                                "
				scanlineMiddle += "         │                                "
				scanlineBottom += "         │                                "
			} else {
				scanlineTop += ui.renderBufferBoxTop(leftBuffer) + "                       "
				scanlineMiddle += ui.renderBufferBoxMiddle(leftBuffer) + "                       "
				scanlineBottom += ui.renderBufferBoxBottom(leftBuffer) + "                       "
			}

			scanlineTop += ui.renderBufferBoxTop(rightBuffer)
			scanlineMiddle += ui.renderBufferBoxMiddle(rightBuffer)
			scanlineBottom += ui.renderBufferBoxBottom(rightBuffer)

			middle = append(middle, scanlineTop)
			middle = append(middle, scanlineMiddle)
			middle = append(middle, scanlineBottom)
			middle = append(middle, "         │                                         │")
			middle = append(middle, "         │                                         ▼")
		}
	}

	bottom := make([]string, 3)

	for i, b := range bl.Bottom {
		if b == -1 && i == 0 {
			bottom[0] += "         │           "
			bottom[1] += "         └───────────"
			bottom[2] += "                     "
			continue
		}

		if b == -1 && i == 1 {
			bottom[0] += "                     "
			bottom[1] += "─────────────────────"
			bottom[2] += "                     "
			continue
		}

		bottom[0] += ui.renderBufferBoxTop(b)
		bottom[1] += ui.renderBufferBoxMiddle(b)
		bottom[2] += ui.renderBufferBoxBottom(b)

		if i < len(bl.Bottom)-1 {
			bottom[0] += "  "
			bottom[1] += "◀─"
			bottom[2] += "  "
		}
	}

	return strings.Join(top, "\n") + "\n" + strings.Join(middle, "\n") + "\n" + strings.Join(bottom, "\n")
}

// The event arrow visually connects the service box to the circuit breaker box
// It visually updates based on the last simulated event and the state of the circuit breaker
func (ui *UI) renderEventArrow() string {
	var colour string
	var eventLabel string
	var stateGate string

	switch ui.es.Event {
	case Fail:
		colour = "\033[31m"
		eventLabel = "Failure"
	case Success:
		colour = "\033[32m"
		eventLabel = "Success"
	case None:
		colour = "\033[0m"
		eventLabel = "   │"
	default:
		colour = "\033[0m"
		eventLabel = "   │"
	}

	switch ui.cb.GetState() {
	case circuitbreaker.Closed:
		stateGate = "│"
	case circuitbreaker.HalfOpen:
		stateGate = "/"
	case circuitbreaker.Open:
		stateGate = "\x1b[0m─"
	}

	a := fmt.Sprintf("                              %s│\033[0m\n", colour)
	a += fmt.Sprintf("                           %s%s\033[0m\n", colour, eventLabel)
	a += fmt.Sprintf("                              %s│\033[0m\n", colour)
	a += fmt.Sprintf("                              %s%s\033[0m\n", colour, stateGate)

	if ui.cb.GetState() != circuitbreaker.Open {
		a += fmt.Sprintf("                              %s│\033[0m\n", colour)
		a += fmt.Sprintf("                              %s▼\033[0m\n", colour)
	} else {
		a += "                              │\n"
		a += "                              ▼\n"
	}

	return a
}

func (ui *UI) stateToString() string {
	state := ui.cb.GetState()

	switch state {
	case circuitbreaker.Open:
		return "\x1b[41m Open \x1b[0m     "
	case circuitbreaker.HalfOpen:
		return "\x1b[43m Half Open \x1b[0m"
	case circuitbreaker.Closed:
		return "Closed"
	default:
		return "Unknown"
	}
}

func (ui *UI) getRetryTime() float64 {
	// Initialize retryStart if not set
	if ui.retryStart.IsZero() {
		ui.retryStart = time.Now().Add(ui.cb.UNSAFEGetRetryTimeout())
	}

	// Calculate remaining time
	remaining := math.Max(0, time.Until(ui.retryStart).Seconds())

	// Reset retryStart if time has elapsed
	if remaining <= 0 {
		ui.retryStart = time.Time{}
	}

	return remaining
}

// The state indicator dynamically shows the most relevant information give the circuit state:
// - Closed Circuit - show how long until the circuit will move to next buffer
// - HalfOpen Circuit - show how many trial successes are still required
// - Open Circuit - show how long until the circuit will move to HalfOpen
func (ui *UI) stateIndicator() string {
	var str string

	cursor := ui.cb.UNSAFEGetActiveCursor()
	state := ui.cb.GetState()
	cursorExpiresIn := math.Max(0, time.Until(cursor.Expires).Seconds())
	trialSuccesses, trialSuccessesRequired := ui.cb.UNSAFEGetTrialState()

	switch state {
	case circuitbreaker.Open:
		str = fmt.Sprintf("                          Retry: %.1fs", ui.getRetryTime())
	case circuitbreaker.Closed:
		str = fmt.Sprintf("                    Next Buffer: %.1fs", cursorExpiresIn)
	case circuitbreaker.HalfOpen:
		str = fmt.Sprintf("                    Trial Success: %d/%d", trialSuccesses, trialSuccessesRequired)
	}

	return str
}

func (ui *UI) Render() string {
	// cursor := ui.cb.UNSAFEGetActiveCursor()

	// state := ui.cb.GetState()
	errorRate := ui.cb.GetErrorRate()
	// cursorExpiresIn := math.Max(0, time.Until(cursor.Expires).Seconds())

	h := `
                       ┌─────────────┐
                       │   Service   │
                       └─────────────┘
`
	h += ui.renderEventArrow()

	h += fmt.Sprintf("                         Status: %s", ui.stateToString()) + "\n"
	h += fmt.Sprintf("                     Error Rate: %.2f%%", errorRate) + "\n"
	// h += fmt.Sprintf("                    Next Buffer: %.1fs", cursorExpiresIn) + "\n"
	h += ui.stateIndicator() + "\n"
	h += ui.renderBufferNodes()
	return h
}
