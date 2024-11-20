class Node {
  constructor(value) {
    this.value = value;
    this.next = null;
  }
}

class RingBuffer {
  #length;
  #cursor;
  #firstNode;

  constructor(elements) {
    this.#length = elements;
    this.#cursor;
    this.#firstNode;

    for (let i = 0; i < elements; i++) {
      let node = new Node(null);

      if (i === 0) {
        this.#firstNode = node;
      } else {
        this.#cursor.next = node;
      }

      this.#cursor = node;

      if (i === elements - 1) {
        this.#cursor.next = this.#firstNode;
      }
    }

    this.#cursor = this.#firstNode;
  }

  get length() {
    return this.#length;
  }

  get cursor() {
    return this.#cursor;
  }

  next() {
    this.#cursor = this.#cursor.next;
  }

  do(fn) {
    let cursor = this.#cursor;
    for (let i = 0; i < this.#length; i++) {
      fn(cursor);
      cursor = cursor.next;
    }
  }
}

class CircuitBreaker {
  static State = {
    Closed: "closed",
    Open: "open",
    HalfOpen: "half_open",
  };

  static Result = {
    Success: "success",
    Failure: "failure",
  };

  #ring;
  #state;
  #date;

  // minimum number of events required in the buffer to evaluate the error rate
  #minEvalSize;

  // percentage of errors that will cause the circuit to Open
  #errorThreshold;

  // millisecond duration the circuit breaker remains in the Open state before transitioning to HalfOpen.
  #retryTimeout;

  // timestamp for when to transition from Open state to HalfOpen
  #retryAfter;

  // millisecond duration of data each node/span in the buffer stores
  #spanDuration;

  // how many successive successes required to close a half open circuit
  #trialSuccessesRequired;

  // how many successive successes have occurred while circuit is HalfOpen
  #trialSuccesses;

  constructor(config = {}) {
    const {
      // minEvalSize sets the minimum number of events required within the evaluation window
      // to assess the error rate and determine the circuit breaker state.
      //
      // If not set, the defaults is 100 events.
      minEvalSize = 100,

      // errorThreshold configures the error rate threshold for the circuit breaker.
      // The threshold is a percentage (0-100) of failed requests relative to total requests.
      // When the error rate exceeds this threshold, the circuit breaker will open.
      //
      // If not set, the default threshold is 10 (10%)
      errorThreshold = 10,

      // retryTimeout configures the duration the circuit breaker remains in the Open state before
      // transitioning to Half-Open. This timeout represents a "cooling off" period, allowing the underlying
      // system time to recover before the circuit breaker cautiously allows traffic through again in the Half-Open state.
      //
      // Note: Setting a very short timeout might lead to rapid oscillation between Open and Half-Open states
      // if the underlying system hasn't fully recovered. Conversely, a very long timeout might
      // unnecessarily delay recovery if the system issues are resolved quickly.
      //
      // If not set, the default is 60000 milliseconds
      retryTimeout = 60000,

      // evalWindow configures the evaluation window for the circuit breaker.
      // The evaluation window determines the duration and granularity of data considered
      // when assessing the circuit breaker state.
      //
      // Fields:
      //   - minutes: The total duration of the evaluation window in minutes.
      //     Defaults to 10 minutes if not specified.
      //   - spans: The number of time spans the evaluation window is divided into.
      //     This allows for more granular data collection and analysis.
      //
      // The circuit breaker requires data for the full evaluation window before making state decisions.
      //
      // Example: evalWindow = {minutes: 10, spans: 5} creates an evaluation window of 10 minutes
      // divided into 5 spans of 2 minutes each.
      evalWindow = { minutes: 10, spans: 5 },

      // trialSuccessesRequired configures the number of consecutive successful requests needed
      // while in the Half-Open state before the circuit breaker transitions back to the Closed state.
      //
      // This acts as a confidence threshold - requiring multiple successful requests helps ensure
      // the underlying system has truly recovered before fully restoring traffic.
      //
      // Note: Setting this value too low might result in premature recovery if the system
      // is still unstable. Setting it too high might unnecessarily delay recovery.
      //
      // If not set, the default is 20 successful requests
      trialSuccessesRequired = 20,

      // UNSAFEDate provides a custom date/time provider for the circuit breaker.
      //
      // This is particularly(only?) useful for unit testing, where you may need to control or simulate time progression.
      //
      // The provider must implement the following interface:
      // {
      //   Now: () => number  // Returns current timestamp in milliseconds
      // }
      //
      // Example:
      //   const testDate = {
      //     Now: () => 1647270000000  // Fixed timestamp for testing
      //   };
      //
      // If not set, the circuit breaker uses the system clock
      UNSAFEDate,
    } = config;

    this.#state = CircuitBreaker.State.Closed;
    this.#minEvalSize = minEvalSize;
    this.#errorThreshold = errorThreshold;
    this.#retryTimeout = retryTimeout;
    this.#trialSuccessesRequired = trialSuccessesRequired;

    // console.log(evalWindow.minutes, evalWindow.spans);
    if (evalWindow.minutes <= 0 || evalWindow.spans <= 0) {
      this.#ring = new RingBuffer(6);
      this.#spanDuration = 120000;
    } else {
      this.#ring = new RingBuffer(evalWindow.spans + 1);
      this.#spanDuration = (evalWindow.minutes / evalWindow.spans) * 60000;
    }

    if (UNSAFEDate) {
      this.#date = UNSAFEDate;
    } else {
      this.#date = Date;
    }
  }

  get state() {
    if (
      this.#state === CircuitBreaker.State.Open &&
      this.#retryAfter < this.#date.now()
    ) {
      this.#state = CircuitBreaker.State.HalfOpen;
    }

    return this.#state;
  }

  clearBuffer() {
    for (let i = 0; i < this.#ring.length; i++) {
      this.#ring.cursor.value = null;
      this.#ring.next();
    }
  }

  record(result) {
    if (this.#state === CircuitBreaker.State.Open) {
      return;
    }

    // If the circuit is HalfOpen, allow a small sample or trial traffic through
    // If 10 consecutive successes occur, assume the service is OK and set the circuit to Closed
    if (
      this.#state === CircuitBreaker.State.HalfOpen &&
      result === CircuitBreaker.Result.Success
    ) {
      this.#trialSuccesses++;
      if (this.#trialSuccesses >= this.#trialSuccessesRequired) {
        this.#state = CircuitBreaker.State.Closed;
      }
      return;
    }

    // If the circuit is HalfOpen, allow a small sample of trial traffic through
    // If an error occurs during the trial, assume the service is still unavailable and set the circuit to Open
    if (
      this.#state === CircuitBreaker.State.HalfOpen &&
      result === CircuitBreaker.Result.Failure
    ) {
      this.#state = CircuitBreaker.State.Open;
      this.#retryAfter = this.#date.now() + this.#retryTimeout;
      this.#trialSuccesses = 0;
      return;
    }

    if (this.#ring.cursor.value === null) {
      this.#ring.cursor.value = {
        expires: this.#date.now() + this.#spanDuration,
        errorCount: 0,
        totalCount: 0,
      };
    }

    if (this.#ring.cursor.value.expires < this.#date.now()) {
      this.#ring.next();
      this.#ring.cursor.value = {
        expires: this.#date.now() + this.#spanDuration,
        errorCount: 0,
        totalCount: 0,
      };
    }

    if (result === CircuitBreaker.Result.Failure) {
      this.#ring.cursor.value.errorCount++;
    }

    this.#ring.cursor.value.totalCount++;

    // If the error rate exceeds the threshold, set the circuit breaker to Open
    const errorRate = this.errorRate;
    if (
      this.#state === CircuitBreaker.State.Closed &&
      errorRate > this.#errorThreshold
    ) {
      this.#state = CircuitBreaker.State.Open;
      this.#retryAfter = this.#date.now() + this.#retryTimeout;
      this.clearBuffer();
    }
  }

  get errorRate() {
    let errors = 0;
    let count = 0;
    let nodes = 0;
    let skipCurrNode = true;

    this.#ring.do(function (node) {
      if (skipCurrNode) {
        skipCurrNode = false;
        return;
      }

      if (
        node.value &&
        node.value.errorCount !== undefined &&
        node.value.totalCount !== undefined &&
        node.value.expires
      ) {
        errors += node.value.errorCount;
        count += node.value.totalCount;
        nodes++;
      }
    });

    if (nodes < this.#ring.length - 1 || count < this.#minEvalSize) {
      return 0;
    }

    const errorRate = Math.round((errors / count) * 100 * 100) / 100;
    return errorRate;
  }
}

module.exports = CircuitBreaker;
