# Breaker Box

A collection of the circuit breaker pattern implemented in different programming languages.

## The pattern

A circuit breaker automatically detects failures in dependencies of a distributed system. When a dependency is detected as failing, the circuit breaker prevents traffic from being sent to the component, while periodically sampling a small percentage of traffic to verify if the dependent service has recovered. This prevents cascading failures that can be caused by continuing to send traffic to a failing service.