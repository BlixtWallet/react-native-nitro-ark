#!/bin/bash

# Exit on error
set -e

# Set up iOS-specific environment
unset PLATFORM_NAME
unset DEVELOPER_DIR
unset SDKROOT
unset IPHONEOS_DEPLOYMENT_TARGET
unset TVOS_DEPLOYMENT_TARGET
unset XROS_DEPLOYMENT_TARGET
unset MACOSX_DEPLOYMENT_TARGET
export PLATFORM_NAME=iphoneos
export DEVELOPER_DIR="$(xcode-select -p)"
export SDKROOT="$DEVELOPER_DIR/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator.sdk"

# First, make sure we have the targets
rustup target add \
    x86_64-apple-ios \
    aarch64-apple-ios \
    aarch64-apple-ios-sim

# Then, build the library
TARGET_DIR="target/ios"
BINARY_NAME="libbark_cpp.a"

mkdir -p $TARGET_DIR

echo "Building for iOS (arm64)..."
cargo build --release \
    --target aarch64-apple-ios \
    --lib \
    --target-dir $TARGET_DIR

# echo "Building for iOS (x86_64)..."
# cargo build --release \
#     --target x86_64-apple-ios \
#     --target-dir "$TARGET_DIR"

echo "Building for iOS (aarch64-sim)..."
cargo build --release \
    --target aarch64-apple-ios-sim \
    --lib \
    --target-dir $TARGET_DIR


# Create temporary directories for the frameworks
mkdir -p target/ios/ios-device/Headers target/ios/ios-simulator/Headers

rm -rf target/Ark.xcframework
HEADERS_DIR_IOS="target/ios/ios-device/Headers"
HEADERS_DIR_IOS_SIM="target/ios/ios-simulator/Headers"

# Create the framework structures
# Create XCFramework for the main library
xcodebuild -create-xcframework \
  -library target/ios/aarch64-apple-ios/release/$BINARY_NAME \
  -headers $HEADERS_DIR_IOS \
  -library target/ios/aarch64-apple-ios-sim/release/$BINARY_NAME \
  -headers $HEADERS_DIR_IOS_SIM \
  -output target/Ark.xcframework

# Copy the XCFramework to the react-native-nitro-ark directory
DEST_XCFRAMEWORK_DIR="../../react-native-nitro-ark/react-native-nitro-ark/Ark.xcframework"
echo "Copying Ark.xcframework to $DEST_XCFRAMEWORK_DIR"
rm -rf "$DEST_XCFRAMEWORK_DIR" # Remove existing framework if any
cp -R "target/Ark.xcframework" "$DEST_XCFRAMEWORK_DIR"

# Also copy the XCFramework to the example app's iOS directory
DEST_XCFRAMEWORK_EXAMPLE_DIR="../../react-native-nitro-ark/react-native-nitro-ark/example/ios/Ark.xcframework"
echo "Copying Ark.xcframework to $DEST_XCFRAMEWORK_EXAMPLE_DIR"
rm -rf "$DEST_XCFRAMEWORK_EXAMPLE_DIR" # Remove existing framework if any
cp -R "target/Ark.xcframework" "$DEST_XCFRAMEWORK_EXAMPLE_DIR"

unset PLATFORM_NAME
unset DEVELOPER_DIR
unset SDKROOT
