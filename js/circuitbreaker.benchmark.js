const Benchmark = require('benchmark');
const { CircuitBreaker } = require('./circuitBreaker.js');

const cb = new CircuitBreaker();

// Pre-populate sample traffic for .record
const isErrors = Array.from({ length: 1000 }, () => Math.random() < 0.1);
let i = 0;

const suite = new Benchmark.Suite()

suite
  .add('CircuitBreakerRecord', function() {
    // [i++ % isErrors.length]
    // i++ returns i before incrementing its value
    // % isErrors.length trick means i will never fall out of bounds of array length
    // eg 101 % 100 = 1, which means when i exceeds array length it will return to array start
    if (isErrors[i++ % isErrors.length]) {
      cb.record(CircuitBreaker.Result.Failure);
    } else {
      cb.record(CircuitBreaker.Result.Success);
    }
  })
  .add('CircuitBreakerGetState', function() {
    cb.state;
  })
  .on('cycle', function(event) {
    const benchmark = event.target;
    const stats = benchmark.stats;
    const meanNs = stats.mean * 1e9;
    
    console.log(benchmark.name)
    console.log(benchmark.count);
    console.log(`${benchmark.hz.toFixed(2)} op/s`);
    console.log(`${meanNs.toFixed(2)} ns/op`);
    console.log(``)
  })
  .run({ 
    'async': true,
    'minTime': 1,
    'maxTime': 5,
    'minSamples': 5
  });