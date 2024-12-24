package circuitbreaker

import (
	"math"
	"time"
)

type Node struct {
	Expires      time.Time
	FailureCount int
	SuccessCount int
}

func (n *Node) Reset(expires time.Time) {
	n.Expires = expires
	n.FailureCount = 0
	n.SuccessCount = 0
}

func (n *Node) Clear() {
	n.Expires = time.Time{}
	n.FailureCount = 0
	n.SuccessCount = 0
}

type RingBuffer struct {
	cursor int
	nodes  []Node
}

func NewRingBuffer(size int) *RingBuffer {
	return &RingBuffer{
		cursor: 0,
		nodes:  make([]Node, size),
	}
}

func (r *RingBuffer) Next() {
	if r.cursor == len(r.nodes)-1 {
		r.cursor = 0
	} else {
		r.cursor += 1
	}
}

func (r *RingBuffer) Len() int {
	return len(r.nodes)
}

func (r *RingBuffer) Cursor() *Node {
	return &r.nodes[r.cursor]
}

func (r *RingBuffer) GetErrorRate(minEvalSize int) float64 {
	failures := 0
	total := 0

	for i, c := range r.nodes {
		if i == r.cursor {
			continue
		}
		failures += c.FailureCount
		total += +c.FailureCount + c.SuccessCount
	}

	if total < minEvalSize || total == 0 {
		return 0
	}

	errorRate := (float64(failures) / float64(total)) * 100
	return math.Round(errorRate*100) / 100
}

func (r *RingBuffer) ClearBuffer() {
	for i := range r.nodes {
		r.nodes[i].Clear()
	}
}
