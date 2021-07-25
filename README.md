# Open Fightstick (OFS)

OFS is an open source framework for using an Arduino UNO as the usb controller for an arcade stick. OFS currently supports an 11 button and 2 axis joystick layout.

## Project Strucure

`usb-firmware/` contains the usb controller that is flashed in DFU-programming mode to be executed on the `atmega16u2` aboard the Arduino UNO.

`controller/` contains the project that is executed on the `atmega328p` aboard the Arduino UNO.

`ofs-support/` contains shared objects between the two projects, such as the fightstick structure and ids for message passing.

`scripts/` contains the cli tool for ofs, written for use with `deno`.

## Getting Started

Note: Until solved within `avr-hal` and `avr-devices`, rust version `nightly-2021-01-07` must be used. See [the avr-hal readme](https://github.com/Rahix/avr-hal/tree/2996b4a7885d40fad544b012f5c6b72e41e35106) for more information.

### Packages Needed
- `deno`
- `rustup`
- `avr-gcc`
- `dfu-programmer`

```
# Set up correct version of Rust Nightly
rustup override set nightly-2021-01-07

## OFS Cli tool (assumed in root directory of project)

# Display subcommands
deno run -A scripts/ofs.ts help

# Build controller
deno run -A scripts/ofs.ts buildc

# Build and flash controller
deno run -A scripts/ofs.ts flashc

# Build usb firmware
deno run -A scripts/ofs.ts buildusb

# Build and flash usb firmware
deno run -A scripts/ofs.ts flashusb

# Restore arduino to standard firwmare for flashing the controller
deno run -A scripts/ofs.ts restoreusb
```

## Modifying the Fightstick
All modification to the fightstick layout can be completed in `controller/src/fightstick.rs`

`fightstick::setup_ports` is used to set up PORTD for whatever I/O layout is required for your given arcade stick.

`fightstick::build_fightstick_data` is used to construct the given input state for the fightstick.

## Message Passing
Message passing between the controller and usb firmware is done via UART. For every message, an acknowledgement is expected in the form of an identical message back, in addition to any other expected data.

The usb firmware, once usb is configured with the host and ample time has passed for the controller to be ready, sends an introductory message (`UsartCommand::Introduction`) to ensure the controller is expecting OFS messages. If an acknowledgement is sent in response, the usb firmware will continuously ask for the state of the fightstick (`UsartCommand::SendData`) and pass the `FightstickDescriptor` response onto the usb host.

## Acknowledgement

Much of the usb firmware is based on configurations used by the [`UnoJoy`](https://github.com/AlanChatham/UnoJoy) project. Special thanks to the maintainers for creating such a usable base to adapt and bring into the Rust ecosystem.
