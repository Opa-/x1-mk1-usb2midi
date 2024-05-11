#!/bin/bash

BUILD_TARGET=$1

# Add build targets
rustup target add $BUILD_TARGET
# Bundle application
cargo install cargo-bundle
cargo bundle --release --target=$BUILD_TARGET
# Copy libusb to the app bundle
LIBUSB_DYLIB_PATH=`otool -L target/$BUILD_TARGET/release/bundle/osx/X1Mk1\ usb2midi.app/Contents/MacOS/x1-mk1-usb2midi | grep libusb | awk -F ' ' '{print $1}'`
LIBUSB_DYLIB_NAME=`basename $LIBUSB_DYLIB_PATH`
echo $LIBUSB_DYLIB_PATH
echo $LIBUSB_DYLIB_NAME
cp $LIBUSB_DYLIB_PATH target/$BUILD_TARGET/release/bundle/osx/X1Mk1\ usb2midi.app/Contents/MacOS/
# Update the libusb path in the app bundle
install_name_tool -change $LIBUSB_DYLIB_PATH @executable_path/$LIBUSB_DYLIB_NAME target/$BUILD_TARGET/release/bundle/osx/X1Mk1\ usb2midi.app/Contents/MacOS/x1-mk1-usb2midi
# Zip the app bundle
cd ./target/$BUILD_TARGET/release/bundle/osx ; zip X1Mk1-usb2midi_$BUILD_TARGET.zip * -r ; cd -
