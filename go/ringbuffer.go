package circuitbreaker

import (
	"time"
)

type Node struct {
	Index        int
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
	nodes := make([]Node, size)
	for i := range nodes {
		nodes[i].Clear()
		nodes[i].Index = i
	}

	return &RingBuffer{
		cursor: 0,
		nodes:  nodes,
	}
}

func (r *RingBuffer) Next() {
	if r.cursor == len(r.nodes)-1 {
		r.cursor = 0
	} else {
		r.cursor += 1
	}
}

func (r *RingBuffer) Cursor() *Node {
	return &r.nodes[r.cursor]
}

// Traverse the ring, starting at the active node
func (r *RingBuffer) Do(f func(*Node)) {
	for i := r.cursor; i < len(r.nodes); i++ {
		f(&r.nodes[i])
	}

	for i := 0; i < r.cursor; i++ {
		f(&r.nodes[i])
	}
}

// Traverse the ring, starting from the head(start)
func (r *RingBuffer) DoFromHead(f func(*Node)) {
	for i := 0; i < len(r.nodes); i++ {
		f(&r.nodes[i])
	}
}

func (r *RingBuffer) ClearBuffer() {
	for i := range r.nodes {
		r.nodes[i].Clear()
	}
}
