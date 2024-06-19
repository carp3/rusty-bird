# Rusty Bird

Weekend project: A Flappy Bird clone written in Rust using the Bevy engine.


Assets are from [here](https://github.com/samuelcust/flappy-bird-assets).


### How to run: 
On you local machine:

`cargo run`

In your browser: 
```
rustup target add wasm32-unknown-unknown 
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-server-runner
cargo run --target wasm32-unknown-unknown
```

License: Public domain