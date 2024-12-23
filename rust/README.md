# Rust Circuit Breaker

```rust
let mut cb = circuit_breaker::CircuitBreaker::new(Settings {
	buffer_size: 5,
	min_eval_size: 1000,
	error_threshold: 40.0,
	retry_timeout: Duration::from_millis(180_000),
	buffer_span_duration: Duration::from_secs(300),
	trial_success_required: 20,
});

fn very_important(cb: &mut CircuitBreaker) -> Result<String, String> {
	match cb.get_state() {
		State::Open(_) => Err("503: Service Unavailable".to_string()),
		_ => match get_critical_data_from_service() {
			Ok(data) => {
				cb.record::<(), ()>(Ok(()));
				Ok(data)
			},
			Err(_) => {
				cb.record::<(), ()>(Err(()));
				Err("500: Internal Server Error".to_string())
			},
		},
	}
}
```
