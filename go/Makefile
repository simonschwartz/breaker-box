test:
	go test

bench:
	go test -bench="BenchmarkCircuit(GetState|BreakerRecord)" -benchmem -v ./...

bench-integration:
	go test -bench="^BenchmarkCircuitBreaker$$" -benchmem -benchtime=30s -v ./...
