import { CircuitBreaker as TSCircuitBreaker } from "./circuitbreaker";
import { CircuitBreaker as JSCircuitBreaker } from "./circuitbreaker.js";
import { strictEqual } from "node:assert/strict";
class MockDate {
  currentTime;

  constructor(mockCurrentTime = Date.now()) {
    this.currentTime = mockCurrentTime;
  }

  now() {
    return this.currentTime;
  }

  fastForward(duration) {
    this.currentTime += duration;
  }
}

function testFunc(CircuitBreaker) {
  function recordErrors(num, cb) {
    for (let i = 0; i < num; i++) {
      cb.record(CircuitBreaker.Result.Failure);
    }
  }

  function recordSuccesses(num, cb) {
    for (let i = 0; i < num; i++) {
      cb.record(CircuitBreaker.Result.Success);
    }
  }

  const mockDate = new MockDate();
  const cb = new CircuitBreaker({
    evalWindow: { minutes: 3, spans: 3 },
    UNSAFEDate: mockDate,
  });

  // First - fill the buffer with events
  const initialEvents = [
    {
      errors: 0,
      successes: 200,
    },
    {
      errors: 22,
      successes: 143,
    },
    {
      errors: 1,
      successes: 100,
    },
    {
      errors: 0,
      successes: 292,
    },
    {
      errors: 5,
      successes: 192,
    },
  ];

  for (let i = 0; i < initialEvents.length; i++) {
    recordErrors(initialEvents[i].errors, cb);
    recordSuccesses(initialEvents[i].successes, cb);
    mockDate.fastForward(61000);
  }

  strictEqual(cb.errorRate, 4.12);
  strictEqual(cb.state, CircuitBreaker.State.Closed);

  // Second - simulate a spike in errors to Open the circuit
  recordErrors(250, cb);
  recordSuccesses(1, cb);
  mockDate.fastForward(61000);
  recordErrors(1, cb);

  strictEqual(cb.errorRate, 0);
  strictEqual(cb.state, CircuitBreaker.State.Open);

  // Third - wait 1 minute for the circuit to move to HalfOpen
  mockDate.fastForward(61000);
  recordSuccesses(1, cb);

  cb.state;
  strictEqual(cb.state, CircuitBreaker.State.HalfOpen);

  // Fourth - oh no, an error, the circuit goes back to Open
  recordErrors(1, cb);
  strictEqual(cb.state, CircuitBreaker.State.Open);

  // Fifth - wait 1 minute for the circuit to move to HalfOpen
  // Add 20 consecutive values so the circuit closes
  mockDate.fastForward(61000);

  cb.state;
  strictEqual(cb.state, CircuitBreaker.State.HalfOpen);

  recordSuccesses(20, cb);

  cb.state;
  strictEqual(cb.state, CircuitBreaker.State.Closed);
}

const test = (desc, circuitBreaker) => {
  try {
    testFunc(circuitBreaker);
    console.log("\x1b[32m%s\x1b[0m", `\u2714 ${desc}`);
  } catch (error) {
    console.log("\n");
    console.log("\x1b[31m%s\x1b[0m", `\u2718 ${desc}`);
    console.error(error);
  }
};

test("TypeScript Circuit Breaker", TSCircuitBreaker);

test("JavaScript Circuit Breaker", JSCircuitBreaker);
