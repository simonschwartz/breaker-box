# Go Circuit Breaker

```go
cb := circuitbreaker.New()

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
