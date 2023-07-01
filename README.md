# oneliner

## About
oneliner is a [Zellij](https://github.com/zellij-org/zellij) plugin which is a modified one-line version of Zellij's default [status-bar](https://github.com/zellij-org/zellij/tree/main/default-plugins/status-bar) plugin. oneliner combines the `firstline` and `secondline` output of the default status-bar into one line.

This is a soft fork of Zellij's default [status-bar](https://github.com/zellij-org/zellij/tree/main/default-plugins/status-bar) plugin.

## Requirements
- [Zellij](https://github.com/zellij-org/zellij): ver >= `0.37.0`
- [Rust](https://www.rust-lang.org/). Recommended to use Rust through [rustup](https://rustup.rs/).

## Usage
### Building from source
To build from source, clone this repository, and then run:
```sh
# Build the plugin
cargo build --release --locked
```

### Install
To install the plugin, run:
```sh
# Get the plugin directory from Zellij's config
zellij setup --check
# Copy the plugin over to the [PLUGIN DIR] specified above
cp /path/to/repo/target/wasm32-wasi/release/oneliner.wasm [PLUGIN DIR]
```

### Setup
To use the plugin, first add it to the list of plugins in your Zellij config:
```
plugins {
    ...
    oneliner { path "oneliner"; }
    ...
}
```
Then, modify your layout appropriately by adding:
```
layout {
    ...
    pane borderless=true size=1 {
        plugin location="zellij:oneliner"
    }
    ...
}
```

### Testing in dev mode
To test the plugin in dev mode:
```sh
# Build the plugin in dev mode
cargo build
# Running in Zellij with provided layout for testing
zellij -l plugin.yaml
```
