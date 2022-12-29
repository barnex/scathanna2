pub fn exit_on_error(result: Result<(), anyhow::Error>) {
	match result {
		Err(e) => {
			eprintln!("ERROR: {e:#}");
			std::process::exit(1)
		}
		Ok(_) => (),
	}
}
