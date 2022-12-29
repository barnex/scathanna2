# Scathanna 2



## Installation

Make sure you have Rust installed from [rustup.rs](http://rustup.rs).

Install dependencies:

```
sudo apt install \
	libfontconfig-dev
	cmake
	libasound2-dev
```

Then

```
git clone https://github.com/barnex/scathanna-v2.git
cd scathanna-v2

cargo run --release --bin server &
cargo run --release --bin play

```

## Settings

Edit your settings in `settings.toml`