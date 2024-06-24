# MCP2518FD

A `#![no_std]` Rust driver library for interacting with [MCP2518FD](https://www.microchip.com/en-us/product/mcp2518fd) CAN FD controller chips. Platform agnostic, specifically tested using the [MCP251863](https://www.microchip.com/en-us/product/mcp251863) chip (MCP2518FD controller with integrated CAN FD transceiver).

This driver attempts to improve on previous such crates and strives to expose as much functionality as possible while making it easy to use for the majority of common use cases.

## Cargo Features

All features are disabled by default.

- `defmt` - Implements `defmt::Format` for most public types so they can be printed using `defmt::info!()` and relatives

## Examples

Examples for the Raspberry Pi Pico (`rp2040` microcontroller) are available in the `examples/`
folder.

- [Raspberry Pi Pico](./examples/rp-pico/)
  - Basic example usage to send a receive messages
- [Raspberry Pi Pico with Interrupts](./examples/rp-pico-interrupts/)
  - More advanced example using interrupts instead of polling

## Current Limitations

The driver does not currently have full support for the usage of the RRS bit in CAN FD standard frames as SID11. This is not a priority since it deviates from the ISO 11898-1:2015 specification. Not supporting this also lets us use the `Id` enum from the `embedded-can` crate instead of implementing one ourselves.

The driver does not currently support graceful abortion of message transmission or resetting FIFOs.

The driver does not currently support a nice way to configure bit timing which is important in order to support a larger amount of use cases. Registers can still be configured manually if this is desired.

The driver does not currently support a convenient way to configure the various types of interrupts. Registers can still be configured manually if this is desired.

## Credits

This driver is loosely based on a previous driver crate for the MCP2517FD which can be found [here](https://github.com/PinballWizards/mcp2517fd), but has been significantly reworked, extended, and updated for `embedded-hal` v1.0.
