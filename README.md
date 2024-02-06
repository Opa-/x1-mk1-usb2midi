# X1 Mk1 USB2MIDI

Native Instruments discontinued the support for the Traktor Kontrol X1 MK1 since MacOS Catalina [see here](https://support.native-instruments.com/hc/en-us/articles/360014900358-Compatibility-of-Native-Instruments-Products-on-macOS). This means that the controller is not recognized by MacOS nor Traktor anymore and it's not possible to use it as a MIDI controller because Native Instruments doesn't want to develop a proper driver.

Note that a nicer way of doing this would be to develop a real driver. But in order to use Apple's [DriverKit](https://developer.apple.com/documentation/driverkit), you need to enroll for Apple Developer Program for $99 a year. 

This program aims to :poop: on NI's planned obsolescence and expensive Apple Dev Program by providing a way to use the X1 MK1 as a MIDI controller with the software of your choice.

Thanks to @joherold repository for all the findings [joherold/traktor_x1](https://github.com/joherold/traktor_x1)

> Disclaimer: This is my first Rust program, use at your own risks :)

So far, it has been tested with 2 X1 MK1 against Traktor Pro 3.11.0 44 on a Apple M2 Pro running MacOS Ventura 13.6.4 (22G513).

## Features

- [x] No limit on the number of controllers _(MIDI virtual ports name are generated using the serial number of the controller)_
- [x] Hotplug support
- [x] Basic LED support
- [x] SHIFT button support
- [ ] HOTCUE button support
- [ ] LED to depends on Traktor state

## Requirements

- [libusb](https://formulae.brew.sh/formula/libusb) (Tested with 1.0.27)

## How to build & run

```sh
brew install libusb
cargo run
```
