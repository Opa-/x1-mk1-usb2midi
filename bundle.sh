#!/bin/bash

# Install libusb
brew install libusb
# Add x86_64 target for older macOS versions
rustup target add x86_64-apple-darwin
# Bundle application
cargo install cargo-bundle
cargo bundle --release
cargo bundle --release --target=x86_64-apple-darwin
# Copy libusb to the app bundle
LIBUSB_DYLIB_PATH=`otool -L target/release/bundle/osx/X1Mk1\ usb2midi.app/Contents/MacOS/x1-mk1-usb2midi | grep libusb | awk -F ' ' '{print $1}'`
LIBUSB_DYLIB_NAME=`basename $LIBUSB_DYLIB_PATH`
echo $LIBUSB_DYLIB_PATH
echo $LIBUSB_DYLIB_NAME
cp $LIBUSB_DYLIB_PATH target/release/bundle/osx/X1Mk1\ usb2midi.app/Contents/MacOS/
cp $LIBUSB_DYLIB_PATH target/x86_64-apple-darwin/release/bundle/osx/X1Mk1\ usb2midi.app/Contents/MacOS/
# Update the libusb path in the app bundle
install_name_tool -change $LIBUSB_DYLIB_PATH @executable_path/$LIBUSB_DYLIB_NAME target/release/bundle/osx/X1Mk1\ usb2midi.app/Contents/MacOS/x1-mk1-usb2midi
install_name_tool -change $LIBUSB_DYLIB_PATH @executable_path/$LIBUSB_DYLIB_NAME target/x86_64-apple-darwin/release/bundle/osx/X1Mk1\ usb2midi.app/Contents/MacOS/x1-mk1-usb2midi
# Zip the app bundle
cd ./target/release/bundle/osx ; zip X1Mk1-usb2midi_aarch64.zip * -r ; cd -
cd ./target/x86_64-apple-darwin/release/bundle/osx ; zip X1Mk1-usb2midi_x86_64.zip * -r ; cd -
