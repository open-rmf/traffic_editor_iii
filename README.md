# Traffic Editor III
Welcome to Traffic Editor III.

# Install dependencies and build

Unfortunately we need a newer Rust than what comes with Ubuntu 20.04.

First make sure you don't have any distro-installed Rust stuff on your machine:
```
sudo apt remove rustc cargo
```

If you don't have it already, install `rustup` from the Rust website: https://www.rust-lang.org/tools/install
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Just select the normal defaults (option 1).
A bunch of stuff will happen. Be sure to close and re-open your terminal afterwards, so that it gets all the new stuff.

Alternatively, if you already have a Rust installation managed by `rustup`, you can just do this to bring it up-to-date: `rustup update`

Next, install some dependencies and the source:
```
sudo apt install lld libxcb-shape0-dev libxcb-xfixes0-dev binaryen
git clone ssh://git@github.com/open-rmf/traffic_editor_iii
cd traffic_editor_iii
cargo build --release
```

# Run it

```
cargo run --release
```

# WASM stuff

OK, this is where things get more complex.
It's currently under heavy development by many people, so this situation will likely become a lot simpler in the near future.

Add some Cargo Dependencies:
```
cargo install wasm-bindgen-cli
cargo install basic-http-server
```

At time of writing (late November 2021), currently you'd need to use the following forks to build a `wasm` executable that avoids trying to calculate shadows, which currently crashes the PBR2 rendering pipeline:
 * `bevy` from https://github.com/mrk-its/bevy  on branch `wgpu-0.11`   only changes are to remove the local `wgpu` crates-io patch and the wgpu log filter in line 78 of `crates/bevy_log/src/lib.rs` to `error` to avoid debugging console text
 * `bevy_egui` from https://github.com/jakobhellermann/bevy_egui  on branch `bevy-pipelined` with a few tweaks to `Cargo.toml` to point at the local `bevy` sibling path and add `render` to the feature list. Not sure if that is needed. Specifically drop in this line: `bevy = { path = "../bevy", default-features = false, features = [`

Once these forks are set up, it's quite magical:

```
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir web target/wasm32-unknown-unknown/release/libtraffic_editor_iii.wasm
cd web
basic-http-server . -a 127.0.0.1:1234
```

There are some helper scripts in `scripts` directory which do a bit more stuff, like further optimization steps to reduce the `.wasm` file size.
