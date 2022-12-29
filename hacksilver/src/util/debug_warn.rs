/// Print huge warning if debugging is enabled.
pub fn debug_warn() {
	#[cfg(debug_assertions)]
	{
		println!(
			r"
*************************************************
  WARNING: debug build, performance will suffer!
  Instead, build with:
  
  	cargo build --release
  
  or run with:
  
  	cargo run --release --bin <server/editor/play>
  
*************************************************
"
		);
		std::thread::sleep(std::time::Duration::from_secs(2));
	}
}
