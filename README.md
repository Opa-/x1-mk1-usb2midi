# X1 Mk1 USB2MIDI

Native Instruments discontinued the support for the Traktor Kontrol X1 MK1 since MacOS Catalina. This means that the controller is not recognized by Traktor anymore and it's not possible to use it as a MIDI controller.

This program aims to :poop: on NI's planned obsolescence by providing a way to use the X1 MK1 as a MIDI controller with the software of your choice.

> Disclaimer: This is my first Rust program, use at your own risks :)

So far, it has been tested with 2 X1 MK1 against Traktor Pro 3.11.0 44 on a Apple M2 Pro running MacOS Ventura 13.6.4 (22G513).

## Features

- [x] No limit on the number of controllers _(MIDI virtual ports name are generated using the serial number of the controller)_
- [x] Hotplug support
- [x] Basic LED support
- [ ] SHIFT button support
- [ ] HOTCUE button support
- [ ] LED to depends on Traktor state

## How to build & run

```sh
cargo run
```
