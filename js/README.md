# JavaScript Circuit Breaker

```js
const cb = new CircuitBreaker()


function VeryImportant() {
    const cState := cb.state

    if(cState !== CircuitBreaker.State.Open)
        try {
            const data = await getCriticalDataFromService();
            cb.record(CircuitBreaker.Result.Success);
            return data;
        } catch (error) {
            cb.record(CircuitBreaker.Result.Failure)
            throw new Error(error);
        }
    } else {
        throw new Error("502: service unavailable")
    }
}
```