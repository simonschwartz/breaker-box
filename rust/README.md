# Rust Circuit Breaker

```rust
use circuitbreakers::{CircuitBreaker, Settings, State};

fn main() -> Result<(), String> {
	let mut cb = CircuitBreaker::new(Settings::default());

	on_request(&mut cb)?;
	Ok(())
}

fn on_request(cb: &mut CircuitBreaker) -> Result<(), String> {
	match cb.get_state() {
		State::Open(_) => Err(String::from("503: Service Unavailable")),
		_ => match get_critical_data_from_service() {
			Ok(data) => {
				cb.record::<(), String>(Ok(data));
				Ok(data)
			},
			Err(error) => {
				cb.record::<(), String>(Err(error.clone()));
				Err(String::from("500: Internal Server Error"))
			},
		},
	}
}

fn get_critical_data_from_service() -> Result<(), String> {
	unimplemented!()
}
```
