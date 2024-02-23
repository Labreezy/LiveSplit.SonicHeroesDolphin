# Livesplit.Dolphin.SA2

An auto splitter for Sonic Adventure 2: Battle played with Dolphin (NTSC-U).


##Usage

First, make sure the PC autosplitter for SA2 is deactivated.

Then, right click on livesplit -> Edit Layout -> + -> Control -> Auto Splitting Runtime and browse for the .wasm file in the releases tab.  

Starting, splitting, and resetting are all manual for now and menus are *untimed* so keep that in mind.

## Compilation

This auto splitter is written in Rust. In order to compile it, you need to
install the Rust compiler: [Install Rust](https://www.rust-lang.org/tools/install).

Afterwards install the WebAssembly target:
```sh
rustup target add wasm32-wasi --toolchain nightly
```

The auto splitter can now be compiled:
```sh
cargo b
```

The auto splitter is then available at:
```
target/wasm32-wasi/release/sonic_suggests_autosplitter.wasm
```

Make sure too look into the [API documentation](https://livesplit.org/asr/asr/) for the `asr` crate.

You can use the [debugger](https://github.com/CryZe/asr-debugger) while
developing the auto splitter to more easily see the log messages, statistics,
dump memory and more.
