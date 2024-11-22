# Go Circuit Breaker

```go
cb := circuitbreaker.
    New().
    SetEvalWindow(5, 5).
    SetMinEvalSize(1000).
    SetErrorThreshold(40.0).
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
