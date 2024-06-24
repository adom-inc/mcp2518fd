# mcp2518fd-rp-pico-interrupts

Example utilizing an MCP2518FD with a Raspberry Pi Pico using interrupts. Based off [rp2040-project-template](https://github.com/rp-rs/rp2040-project-template).

## Usage

1. Install dependencies

```
rustup target install thumbv6m-none-eabi
cargo install flip-link
cargo install probe-run
```

2. Run `cargo run` to build and flash the firmware to the Raspberry Pi Pico
