# X1 Mk1 USB2MIDI

[![Rust](https://github.com/Opa-/x1-mk1-usb2midi/actions/workflows/rust.yml/badge.svg)](https://github.com/Opa-/x1-mk1-usb2midi/actions/workflows/rust.yml)

<a href='https://ko-fi.com/opa_sdc' target='_blank'><img height='35' style='border:0px;height:46px;' src='https://az743702.vo.msecnd.net/cdn/kofi3.png?v=0' border='0' alt='Buy Me a Coffee at ko-fi.com' />

Native Instruments discontinued the support for the Traktor Kontrol X1 MK1 since MacOS Catalina [see here](https://support.native-instruments.com/hc/en-us/articles/360014900358-Compatibility-of-Native-Instruments-Products-on-macOS). This means that the controller is not recognized by MacOS nor Traktor anymore and it's not possible to use it as a MIDI controller because Native Instruments doesn't want to develop a proper driver.

Note that a nicer way of doing this would be to develop a real driver. But in order to use Apple's [DriverKit](https://developer.apple.com/documentation/driverkit), you need to enroll for Apple Developer Program for $99 a year. 

This program aims to :poop: on NI's planned obsolescence and expensive Apple Dev Program by providing a way to use the X1 MK1 as a MIDI controller with the software of your choice.

Thanks to @joherold repository for all the findings [joherold/traktor_x1](https://github.com/joherold/traktor_x1)

> Disclaimer: This is my first Rust program, use at your own risks :)

So far, it has been tested with 2 X1 MK1 against Traktor Pro 3.11.0 44 on a Apple M2 Pro running MacOS Ventura 13.6.4 (22G513).

## Install (Linux)

This tool can be used to run the X1 MK1 with Linux.
It has been successfully tested with [Mixxx](https://mixxx.org).
In Linux, this tool does not have a GUI, yet.

* Clone this repository
* Make sure you have installed the [libusb](https://libusb.info/) headers which are necessary for the [rusb](https://github.com/a1ien/rusb) crate. On OpenSUSE Tumbleweed the necessary package was `libusb-devel`.
* Put the following line in a new udev rule, e.g. in `/etc/udev/rules.d/99-x1mk1.rules` so you can access the device without needing to run this as root and also to unload the caiaq driver.
```
SUBSYSTEMS=="usb", ATTRS{idVendor}=="17cc", ATTRS{idProduct}=="2305", ACTION=="add", DRIVER=="snd-usb-caiaq", MODE="0666", RUN+="/bin/sh -c 'echo $kernel > /sys/bus/usb/drivers/snd-usb-caiaq/unbind'"
```
* Build and run with `cargo run`
* Startup Mixxx, it should detect the device, select the X1 mappings (which are not perfectly great yet).

## Install (macOS)

Grab the [latest release](https://github.com/Opa-/x1-mk1-usb2midi/releases/latest) and run the App, it should launch in the Dock and you should have an icon in the menu bar as well, listing all currently connected X1 Mk1.

> The first time you run the app, macOS will not run it, saying it's from an unidentified developer. You need to go in System Settings > Privacy & Security, scroll down and click "Open Anyway".

![](screenshot-tray.webp)

A basic mapping is also available on the release page `DeckAB.tsi`. You can import it into Traktor (created with Traktor 3, not sure about Traktor 2 compatibility). The application creates virtual MIDI ports using the serial number of the controller as the name so you should put it as both `Input` and `Output` like this :

![](screenshot-traktor-in-out.webp)

If you need additional controller, just duplicate the device, assign the `In-Port` & `Out-Port` to the other X1 Mk1 serial number and click on `Edit` -> `AB > CD` to convert the mapping :

![](screenshot-traktor-abcd-decks.webp)

Here's [a video](https://streamable.com/ziu6pa) if needed for the later step.

If you want to do your own mapping, you can simply use the "Learn" feature of Traktor and click on the buttons 😉

**🫵 Your feedback is essential to enhance/fix this app. Feel free to report any feedback or any issue you could encounter in the [discussions tab](https://github.com/Opa-/x1-mk1-usb2midi/discussions) or the [issues tab](https://github.com/Opa-/x1-mk1-usb2midi/issues).**

## Known issues

- The `<| BEAT` and `BEAT |>` buttons can only be used as "Hold" button types. All other buttons can only be used as "Toggle" button types.

## Features

- [x] No limit on the number of controllers
- [x] Hotplug support
- [x] LED support
- [x] SHIFT button support
- [X] HOTCUE button support
- [X] macOS menu bar integration

## Roadmap

- [ ] Document which MIDI channels and CC are used for each knob/button/encoder
- [x] Linux
- [ ] Windows 11
- [ ] GUI configuration

# Contributing

Feel free to contribute to the project by posting an issue, a thread in the discussions tab or directly a PR.

## Development Requirements

- [libusb](https://libusb.info/) (Tested with 1.0.27)
