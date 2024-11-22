<pre align="center" aria-label="Breaker Box">
┌─────────────────────────┐
│ ○                     ○ │
│   ┌───┐  ┌───┐  ┌───┐   │
│  ┌┴───┴┐┌┴───┴┐┌┴───┴┐  │
│  │  ⚡  ││  ⚡  ││  ⚡  │  │
│  │ ┌─┐ ││┌───┐││ ┌─┐ │  │
│  │ │ │ ││└┬─┬┘││ │ │ │  │
│  │┌┴─┴┐││ │ │ ││┌┴─┴┐│  │
│  │└───┘││ └─┘ ││└───┘│  │
│  └┬───┬┘└┬───┬┘└┬───┬┘  │
│   └───┘  └───┘  └───┘   │
│ ○                     ○ │
└────┬─┬────┬─┬────┬─┬────┘
     └┬┘    └┬┘    └┬┘     
      └┐     ╲     ┌┘      
┌──────┴─────┴─────┴──────┐
│       Breaker Box       │
└─────────────────────────┘
</pre>

A collection of the circuit breaker pattern implemented in different programming languages.

## Pattern

A circuit breaker automatically detects failures in dependencies of a system. When a dependency is detected as failing, the circuit breaker prevents traffic from being sent to the service, while periodically sampling a small percentage of traffic to verify if the dependent service has recovered. This prevents cascading failures that can be caused by continuing to send traffic to a failing service.

## Approach
                                                                          
                                                                          
    ┌───────────────────┐   ┌───────────────────┐   ┌───────────────────┐ 
    │                   │   │                   │   │                   │ 
    │      Request      │   │  Circuit Breaker  │   │      Service      │ 
    │                   │   │                   │   │                   │ 
    └───────────────────┘   └───────────────────┘   └───────────────────┘ 
              │                       │                       │           
              │                       │                       │           
     Normal Closed State ─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ┐
              │                       │                       │           
     │        │                       │   ┌─────────────┐     │          │
              │                       │   │   State =   │     │           
     │        ├──────────────────────▶│─ ─│   Closed    │     │          │
              │                       │   └─────────────┘     │           
     │        │                       │                       │          │
              │                       ├──────────────────────▶│           
     │        │                       │                       │          │
              │                       │        Response       │           
     │        │                       │◀──────────────────────┤          │
              │                       │                       │           
     │        │        Response       │                       │          │
              │◀──────────────────────┤                       │           
     │        │                       │                       │          │
              │                       │                       │           
     │        │                       │                       │          │
      ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ 
              │                       │                       │           
     Failing Closed State─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ┐
              │                       │                       │           
     │        │                       │   ┌─────────────┐     │          │
              │                       │   │   State =   │     │           
     │        ├──────────────────────▶│─ ─│   Closed    │     │          │
              │                       │   └─────────────┘     │           
     │        │                       │                       │          │
              │                       ├──────────────────────▶│           
     │        │                       │                       │          │
              │                       │         Error         │           
     │        │                       │◀──────────────────────┤          │
              │                       │─                      │           
     │        │         Error         │ │  ┌─────────────┐    │          │
              │◀──────────────────────┤    │  Increment  │    │           
     │        │                       │ └ ─│ Error Count │    │          │
              │                       │    └─────────────┘    │           
     │        │                       │                       │          │
      ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ 
              │                       │                       │           
     Transition Open State ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ┐
              │                       │                       │           
     │        │                       │   ┌─────────────┐     │          │
              │                       │   │   State =   │     │           
     │        ├──────────────────────▶│─ ─│   Closed    │     │          │
              │                       │   └─────────────┘     │           
     │        │                       │                       │          │
              │                       ├──────────────────────▶│           
     │        │                       │                       │          │
              │                       │         Error         │           
     │        │                       │◀──────────────────────┤          │
              │                       │─                      │           
     │        │      Unavailable      │ │  ┌──────────────────┴─────┐    │
              │◀──────────────────────┤    │Error Count > Threshold?│     
     │        │                       │ └ ─│    Set State = Open    │    │
              │                       │    │      Set Timeout       │     
     │        │                       │    └──────────────────┬─────┘    │
      ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ 
              │                       │                       │           
     Open State─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ┐
              │                       │                       │           
     │        │                       │   ┌─────────────┐     │          │
              │                       │   │   State =   │     │           
     │        ├──────────────────────▶│─ ─│    Open     │     │          │
              │                       │   └─────────────┘     │           
     │        │      Unavailable      │                       │          │
              │◀──────────────────────┤                       │           
     │        │                       │                       │          │
              │                       │                       │           
     │        │                       │                       │          │
      ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ 
              │                       │                       │           
     Transition Half Open State─ ─ ─ ─│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ┐
              │                       │                       │           
     │        │                       │   ┌───────────────────┴──┐       │
              │                       │   │     State = Open     │        
     │        ├──────────────────────▶│─ ─│  Timeout = Expired   │       │
              │                       │   │Set State = Half Open │        
     │        │                       │   └───────────────────┬──┘       │
              │                       ├──────────────────────▶│           
     │        │                       │                       │          │
              │     ┌ ─ ─ ─ ─ ─ ─ ─ ─ ┴ ─ ─ ─ ─ ─ ─ ─ ─ ┐     │           
     │        │                                               │          │
              │     │   Sample traffic to evaluate if   │     │           
     │        │         service is still unavailable.         │          │
              │     │        See below examples         │     │           
     │        │                                               │          │
              │     └ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ┘     │           
     │        │                       │                       │          │
      ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ 
              │                       │                       │           
     Half Open State Fail─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ┐
              │                       │                       │           
     │        │                       │   ┌─────────────┐     │          │
              │                       │   │   State =   │     │           
     │        ├──────────────────────▶│─ ─│  Half Open  │     │          │
              │                       │   └─────────────┘     │           
     │        │                       │                       │          │
              │                       ├──────────────────────▶│           
     │        │                       │                       │          │
              │                       │         Error         │           
     │        │                       │◀──────────────────────┤          │
              │                       │─                      │           
     │        │      Unavailable      │ │  ┌────────────────┐ │          │
              │◀──────────────────────┤    │Set State = Open│ │           
     │        │                       │ └ ─│  Set Timeout   │ │          │
              │                       │    └────────────────┘ │           
     │        │                       │                       │          │
      ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ 
              │                       │                       │           
     Half Open State Success ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ┐
              │                       │                       │           
     │        │                       │   ┌─────────────┐     │          │
              │                       │   │   State =   │     │           
     │        ├──────────────────────▶│─ ─│  Half Open  │     │          │
              │                       │   └─────────────┘     │           
     │        │                       │                       │          │
              │                       ├──────────────────────▶│           
     │        │                       │                       │          │
              │                       │        Response       │           
     │        │                       │◀──────────────────────┤          │
              │                       │─   ┌──────────────────┴────────┐  
     │        │       Response        │ │  │   Increment Consecutive   │ │
              │◀──────────────────────┤    │       Success Count       │  
     │        │                       │ └ ─│  Success threshold met?   │ │
              │                       │    │    Set State = Closed     │  
     │        │                       │    └──────────────────┬────────┘ │
      ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┬ ─ ─ ─ ─ ─ 
              │                       │                       │           
              │                       │                       │           
    ┌───────────────────┐   ┌───────────────────┐   ┌───────────────────┐ 
    │                   │   │                   │   │                   │ 
    │      Request      │   │  Circuit Breaker  │   │      Service      │ 
    │                   │   │                   │   │                   │ 
    └───────────────────┘   └───────────────────┘   └───────────────────┘ 

*Notes on Half Open behavior:* For our implementation, we transition the circuit breaker from Open to HalfOpen after a set duration the circuit has been open. When in the HalfOpen state, the circuit routes traffic to the dependent service. If N number of consecutive successes occur, the circuit will move to Closed. If any errors occur the circuit will move back to Open and the wait duration for the next HalfOpen attempt is reset.

## Implementation

I wasn't satisfied with existing circuit breaker implementations as their approaches for detecting if a service was unavailable were too simplistic eg using a fixed error count threshold or time since last successful response.

For this implementation my goals were to be able to:
- intelligently distinguish between error increases due to traffic spikes and genuine service degradation
- respond quickly to service degradations
- configure the sensitivity of what causes the circuit breaker to trip. For example a 5% error rate in a service may want to trip the circuit, however a less critical service may want to wait until a 40% error rate.

### Approach

- Record events within a configurable time window
- Automatically expire data as the time window shifts
- This sliding window approach means we can detect changes quickly and accurately, as we are always retaining the short term history data
- The circuit breaker may need to process and store hundreds of thousands of events for high traffic services, so the solution must be optimised

#### Time Based Ring Buffer

                                                              
       Active Cursor ─ ─ ─ ─ ┐       Node                     
         ┌─────────────────┐         ┌─────────────────┐      
       │ │ ErrorCount int  │ │       │ ErrorCount int  │     │
         │ TotalCount int  │────────▶│ TotalCount int  │     │
       │ │ Expires time    │ │       │ Expires time    │     │
         └─────────────────┘         └─────────────────┘     │
       └ ─ ─ ─ ─ ─▲─ ─ ─ ─ ─ ┘                │              │
                  │                           │              │
         Node     │                  Node     ▼              │
     ▲   ┌─────────────────┐         ┌─────────────────┐     │
     │   │ ErrorCount int  │         │ ErrorCount int  │     │
     │   │ TotalCount int  │         │ TotalCount int  │     │
     │   │ Expires time    │         │ Expires time    │     │
     │   └─────────────────┘         └─────────────────┘     │
     │            ▲                           │              │
     │            │                           │              │
     │   Node     │                  Node     ▼              │
     │   ┌─────────────────┐         ┌─────────────────┐     │
     │   │ ErrorCount int  │         │ ErrorCount int  │     │
     │   │ TotalCount int  │  ◀───── │ TotalCount int  │     │
     │   │ Expires time    │         │ Expires time    │     │
     │   └─────────────────┘         └─────────────────┘     │
     │                                                       │
     └──────────────────Evaluation Window────────────────────┘
                                                              

The approach I chose was to use a fixed size, time based ring buffer. A ring buffer is a linked list, where the last item(tail) has a pointer to the head. In most cases we can make use of languages inbuilt libraries for achieving this, such as Go’s standard library `container/ring`.

The active cursor is used to store incoming events, and calculating the circuit breaker state is done by traversing all nodes, excluding the active cursor, to calculate the overall error rate based on recent history of traffic.

The fixed size, time based ring buffer is a great approach for performance. In the above diagram, updating the circuit breaker state can be done in constant time O(1). Traversing the nodes in the buffer operates in linear time O(n), however this is mitigated by the fact that the size of the ring buffer will be small. There are limited use cases where the ring would exceed 10 nodes. The ring maintains a fixed size, and incoming data simply overwrites expired data, meaning memory will not increase with additional traffic.
