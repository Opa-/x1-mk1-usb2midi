#!/bin/bash

# Bundle application
cargo bundle --release
# Copy libusb to the app bundle
LIBUSB_DYLIB_PATH=`otool -L target/release/bundle/osx/X1Mk1\ usb2midi.app/Contents/MacOS/x1-mk1-usb2midi | grep libusb | awk -F ' ' '{print $1}'`
LIBUSB_DYLIB_NAME=`basename $LIBUSB_DYLIB_PATH`
cp $LIBUSB_DYLIB_PATH target/release/bundle/osx/X1Mk1\ usb2midi.app/Contents/MacOS/
# Update the libusb path in the app bundle
install_name_tool -change $LIBUSB_DYLIB_PATH @executable_path/$LIBUSB_DYLIB_NAME target/release/bundle/osx/X1Mk1\ usb2midi.app/Contents/MacOS/x1-mk1-usb2midi
