#!/bin/bash

BUILD_TARGET=$1
OS_VERSION=$2

APP_NAME="X1Mk1 usb2midi"
APP_BUNDLE_PATH="target/$BUILD_TARGET/release/bundle/osx/$APP_NAME.app"

rustup target add $BUILD_TARGET

cargo install cargo-bundle
cargo bundle --release --target=$BUILD_TARGET

LIBUSB_DYLIB_PATH=`otool -L "$APP_BUNDLE_PATH/Contents/MacOS/x1-mk1-usb2midi" | grep libusb | awk -F ' ' '{print $1}'`
LIBUSB_DYLIB_NAME=`basename $LIBUSB_DYLIB_PATH`

echo "Processing libusb: $LIBUSB_DYLIB_PATH"

cp $LIBUSB_DYLIB_PATH "$APP_BUNDLE_PATH/Contents/MacOS/"

install_name_tool -change $LIBUSB_DYLIB_PATH @executable_path/$LIBUSB_DYLIB_NAME "$APP_BUNDLE_PATH/Contents/MacOS/x1-mk1-usb2midi"

echo "Self-signing app"
codesign --force --deep --sign - "$APP_BUNDLE_PATH"

cd ./target/$BUILD_TARGET/release/bundle/osx


ZIP_NAME="X1Mk1-usb2midi_${BUILD_TARGET}_${OS_VERSION}.zip"
zip "$ZIP_NAME" * -r

cd -
