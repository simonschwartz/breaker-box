# JavaScript Circuit Breaker

```js
const cb = new CircuitBreaker({
  evalWindow: { minutes: 5, spans: 5 },
  minEvalSize: 1000,
  errorThreshold: 40,
  trialSuccessesRequired: 20,
  retryTimeout: 180000,
});

async function veryImportant() {
  const cState = cb.state;

  if (cState !== CircuitBreaker.State.Open) {
    try {
      const data = await getCriticalDataFromService();
      cb.record(CircuitBreaker.Result.Success);
      return data;
    } catch (error) {
      cb.record(CircuitBreaker.Result.Failure);
      throw new Error("500: Internal Server Error");
    }
  } else {
    throw new Error("503: Service Unavailable");
  }
}
```
