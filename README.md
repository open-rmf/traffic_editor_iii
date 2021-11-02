# Traffic Editor III
Welcome to Traffic Editor III.

# install stuff

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
A bunch of stuff will happen. Be sure to reload your `~/.bashrc` afterwards or close and re-open your terminal.

Alternatively, if you already have a Rust installation managed by `rustup`, you can just do this to bring it up-to-date: `rustup update`

Next, install some dependencies and the source:
```
sudo apt install lld libxcb-shape0-dev libxcb-xfixes0-dev binaryen
git clone ssh://git@github.com/open-rmf/traffic_editor_iii
cd traffic_editor_iii
cargo build --release
```

Dependencies.... still figuring this out, and it's changing daily, but currently as siblings to this repo:
 * `bevy` from https://github.com/mrk-its/bevy  on branch `wgpu-0.11`   only changes are to remove the local `wgpu` crates-io patch and the wgpu log filter in line 78 of `crates/bevy_log/src/lib.rs` to `error` to avoid debugging console text
 * `bevy_egui` from https://github.com/jakobhellermann/bevy_egui  on branch `bevy-pipelined` with a few tweaks to `Cargo.toml` to point at the local `bevy` sibling path and add `render` to the feature list. Not sure if that is needed. Specifically drop in this line: `bevy = { path = "../bevy", default-features = false, features = [`

# run stuff

```
cargo run --release
```

# WASM stuff

Dependencies:
```
cargo install wasm-bindgen-cli
cargo install basic-http-server
```

At time of writing, the latest wgpu (v0.11) can support WebGL2. At time of writing, this requires the local clones and slight tweaks to the forks described above, but once this is setup, it's quite magical:
```
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir web target/wasm32-unknown-unknown/release/libtraffic_editor_iii.wasm
cd web
basic-http-server . -a 127.0.0.1:1234
```
