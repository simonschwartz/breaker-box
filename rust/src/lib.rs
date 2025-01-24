//! ```skip
//!  ╔═╗ ╦ ╦═╗ ╔═╗ ╦ ╦ ╦ ╔╦╗      ╔╗  ╦═╗ ╔═╗ ╔═╗ ╦╔═ ╔═╗ ╦═╗
//!  ║   ║ ╠╦╝ ║   ║ ║ ║  ║       ╠╩╗ ╠╦╝ ║╣  ╠═╣ ╠╩╗ ║╣  ╠╦╝
//!  ╚═╝ ╩ ╩╚═ ╚═╝ ╚═╝ ╩  ╩       ╚═╝ ╩╚═ ╚═╝ ╩ ╩ ╩ ╩ ╚═╝ ╩╚═
//! ```
//!
//! > A zero dependencies, rust, circuit breaker implmentation via a ring buffer
//! with time-based rollover.
//!
//! The intention is to give a failing system a break so it can recover.
//!
//! The circuit breaker records the results of each requests into a ring buffer
//! that has `Settings.buffer_size` nodes. Each [Node] is used for
//! `Settings.buffer_span_duration` amount of time to record into before moving
//! to the next [Node]. Each time we move to a new [Node] we check the error
//! rate to be below the `Settings.error_threshold` as long as the nodes contain
//! at least `Settings.min_eval_size` events.
//! If the error rate is above the threshold we set the [State] of the
//! [CircuitBreaker] to `Open`. The open state will ignore all events.
//! After the duration of `Settings.retry_timeout` we set the [State] to
//! `HalfOpen` which means all events are recorded again. If we count at least
//! `Settings.trial_success_required` successful events in succession we set the
//! circuit to `Closed` again. If we encounter a failed event during that time
//! we set the circuit to `Open` again and wait for `Settings.retry_timeout`.
//!
//! Checking for the state of the [CircuitBreaker] allows userland to decide
//! what to do.
//!
//! 💡 This implementation is not thread-safe and should be wrapped in a Mutex if
//! used in a mutli-thread context.
//!
//! ```rust
//! use circuitbreakers::{CircuitBreaker, Settings, State};
//!
//! fn main() -> Result<(), String> {
//!     let mut cb = CircuitBreaker::new(Settings::default());
//!
//!     on_request(&mut cb)?;
//!     Ok(())
//! }
//!
//! fn on_request(cb: &mut CircuitBreaker) -> Result<(), String> {
//!     match cb.get_state() {
//!         State::Open(_) => Err(String::from("503: Service Unavailable")),
//!         _ => match get_critical_data_from_service() {
//!             Ok(data) => {
//!                 cb.record::<(), String>(Ok(data));
//!                 Ok(data)
//!             },
//!             Err(error) => {
//!                 cb.record::<(), String>(Err(error.clone()));
//!                 Err(String::from("500: Internal Server Error"))
//!             },
//!         },
//!     }
//! }
//!
//! fn get_critical_data_from_service() -> Result<(), String> {
//!     // your request logic here
//!     Ok(())
//! }
//! ```

pub mod circuit_breaker;
pub mod ring_buffer;

pub use circuit_breaker::{CircuitBreaker, Settings, State};
pub use ring_buffer::{Node, NodeInfo, RingBuffer};
