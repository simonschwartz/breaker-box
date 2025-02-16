# Go Circuit Breaker

```go
cb := circuitbreaker.
    New().
    SetEvalWindow(5, 5).
    SetMinEvalSize(1000).
    SetErrorThreshold(40.0).
    SetTrialSuccessesRequired(20).
    SetRetryTimeout(3 * time.Minute).
    Build()

func VeryImportant() (*CriticalData, error) {
    cState := cb.GetState()

    if cState != circuitbreaker.Open {
        res, err := getCriticalDataFromService()

        if err != nil {
            cb.Record(circuitbreaker.Failure)
            return nil, err
        } else {
            cb.Record(circuitbreaker.Success)
        }

        return res, nil
    } else {
        return nil, errors.New("503: Service Unavailable")
    }
}
```

### Reporting and debugging

Use `Inspect()` to inspect the current status and configuration of the circuit breaker. Use `Inspect()` sparingly in production as it traverses the entire buffer to gather data and this data infrequently changes.

TODO - Add pub/sub mechanism for registering handlers when state changes occur

### Benchmarks

Run benchmarks on the critical circuit breaker methods:

```
make bench
```

This includes the GetState() method for checking circuit breaker state, and Record() for handling incoming events.

Run extended benchmarks(30 seconds) that simulate real-world usage:

```
make bench-integration
```

### Visualiser

Start the interactive visualiser tool to test the circuit breaker in your CLI

```
go run cmd/vis/vis.go [optional flags]

Flags:
  -d, --duration int            The total duration of the circuit breaker evaluation window in minutes. Defaults to 10 minutes. (default 10)
  -e, --error-threshold float   The error rate threshold that will cause the circuit breaker to open. Defaults to 10.0. (default 10)
  -h, --help                    help for vis
  -m, --min-eval-size int       The minimum number of events required within the evaluation window to assess the error rate and determine the circuit breaker state. Defaults to 100. (default 100)
  -r, --retry int               The duration in seconds the circuit breaker remains in the Open state before transitioning to Half-Open. Defaults to 60. (default 100)
  -s, --spans int               The number of time spans the evaluation window is divided into. This allows for more granular data collection and analysis. Defaults to 10. (default 10)
  -t, --trials int              The number of consecutive successful requests needed while in the Half-Open state before the circuit breaker transitions back to the Closed state. Defaults to 20. (default 20)
```
