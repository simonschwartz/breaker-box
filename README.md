# Breaker Box

A collection of the circuit breaker pattern implemented in different programming languages.

## The pattern

A circuit breaker automatically detects failures in dependencies of a system. When a dependency is detected as failing, the circuit breaker prevents traffic from being sent to the service, while periodically sampling a small percentage of traffic to verify if the dependent service has recovered. This prevents cascading failures that can be caused by continuing to send traffic to a failing service.

## The approach

```mermaid
sequenceDiagram
    participant Request
    participant Circuit Breaker
    participant Service
    
    rect rgb(240, 240, 240)
    Note over Request,Service: Normal Closed State
    Request->>Circuit Breaker: request
    Note over Circuit Breaker: State = Closed
    Circuit Breaker->>Service: request
    Service-->>Circuit Breaker: response
    Circuit Breaker-->>Request: response
    end

    rect rgb(240, 240, 240)
    Note over Request,Service: Failing Closed State
    Request->>Circuit Breaker: request
    Note over Circuit Breaker: State = Closed
    Circuit Breaker->>Service: request
    Service-->>Circuit Breaker: error
    Note over Circuit Breaker: Increment error count
    Circuit Breaker-->>Request: error
    end

    rect rgb(240, 240, 240)
    Note over Request,Service: Transition Open State
    Request->>Circuit Breaker: request
    Note over Circuit Breaker: State = Closed
    Circuit Breaker->>Service: request
    Service-->>Circuit Breaker: error
    Note over Circuit Breaker: Error threshold exceeded.<br/>Set State = Open.<br/>Set timeout
    Circuit Breaker-->>Request: service unavailable response
    end

    rect rgb(240, 240, 240)
    Note over Request,Service: Open State
    Request->>Circuit Breaker: request
    Note over Circuit Breaker: State = Open
    Circuit Breaker-->>Request: service unavailable response
    end

    rect rgb(240, 240, 240)
    Note over Request,Service: Transition into Half Open State
    Request->>Circuit Breaker: request
    Note over Circuit Breaker: State = Open<br/> Timeout = expired
    Circuit Breaker->>Service: request
    Service-->>Circuit Breaker: response
    Note over Circuit Breaker: Set State = HalfOpen<br/>Increment success count
    Circuit Breaker-->>Request: response
    end

    rect rgb(240, 240, 240)
    Note over Request,Service: Transition out of Half Open State to Open
    Request->>Circuit Breaker: request
    Note over Circuit Breaker: State = HalfOpen<br/> Timeout = expired
    Circuit Breaker->>Service: request
    Service-->>Circuit Breaker: error
    Note over Circuit Breaker: Set State = Open<br/>Set timeout
    Circuit Breaker-->>Request: error
    end

    rect rgb(240, 240, 240)
    Note over Request,Service: Transition out of Half Open State to Closed
    Request->>Circuit Breaker: request
    Note over Circuit Breaker: State = HalfOpen<br/> SuccessCount == Required consecutive successes?
    Circuit Breaker->>Service: request
    Service-->>Circuit Breaker: response
    Note over Circuit Breaker: Set State = Closed
    Circuit Breaker-->>Request: response
    end
```

*Notes on Half Open behavior:* For our implementation, we transition the circuit breaker from Open to HalfOpen after a set duration the circuit has been open. When in the HalfOpen state, the circuit routes traffic to the dependent service. If N number of consecutive successes occur, the circuit will move to Closed. If any errors occur the circuit will move back to Open and the wait duration for the next HalfOpen attempt is reset.