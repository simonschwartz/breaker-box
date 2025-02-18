package ringbuffer

type RingBuffer[T any] struct {
	cursor int
	nodes  []T
}

func New[T any](size int) *RingBuffer[T] {
	nodes := make([]T, size)

	return &RingBuffer[T]{
		cursor: 0,
		nodes:  nodes,
	}
}

func (r *RingBuffer[T]) Next() {
	if r.cursor == len(r.nodes)-1 {
		r.cursor = 0
	} else {
		r.cursor += 1
	}
}

func (r *RingBuffer[T]) Cursor() *T {
	return &r.nodes[r.cursor]
}

// Traverse the ring, starting at the active node
func (r *RingBuffer[T]) Do(f func(*T)) {
	for i := r.cursor; i < len(r.nodes); i++ {
		f(&r.nodes[i])
	}

	for i := 0; i < r.cursor; i++ {
		f(&r.nodes[i])
	}
}

// Traverse the ring, starting from the head(start)
func (r *RingBuffer[T]) DoFromHead(f func(*T)) {
	for i := 0; i < len(r.nodes); i++ {
		f(&r.nodes[i])
	}
}
