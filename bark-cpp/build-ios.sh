#!/bin/bash

# Exit on error
set -e

# Unset any iOS/macOS specific variables that might interfere
unset SDKROOT
unset PLATFORM_NAME
unset IPHONEOS_DEPLOYMENT_TARGET
unset TVOS_DEPLOYMENT_TARGET
unset XROS_DEPLOYMENT_TARGET
export PLATFORM_NAME=iphoneos
export DEVELOPER_DIR="$(xcode-select -p)"

# --- Configuration ---
BUILD_TYPE="release"
CARGO_FLAG="--release"

if [ "$1" == "--debug" ]; then
  echo "Performing a debug build."
  BUILD_TYPE="debug"
  CARGO_FLAG=""
else
  echo "Performing a release build."
fi

# IMPORTANT: Change this to the name of your crate as defined in your Cargo.toml
CRATE_NAME="bark-cpp"
TARGET_DIR="target/ios"
BINARY_NAME="libbark_cpp.a"
FRAMEWORK_NAME="Ark.xcframework"

# --- Clean only the specific package artifacts ---
echo "Cleaning previous build artifacts for '$CRATE_NAME'..."
cargo clean --target-dir "$TARGET_DIR" -p "$CRATE_NAME"

# --- Install Rust targets ---
echo "Ensuring required Rust targets are installed..."
rustup target add \
    aarch64-apple-ios \
    aarch64-apple-ios-sim \
    x86_64-apple-ios

# --- Build for each target architecture ---

echo "Building for iOS Device (aarch64-apple-ios)..."
cargo build $CARGO_FLAG \
    --target aarch64-apple-ios \
    --lib \
    --target-dir "$TARGET_DIR"

echo "Building for Apple Silicon Simulator (aarch64-apple-ios-sim)..."
cargo build $CARGO_FLAG \
    --target aarch64-apple-ios-sim \
    --lib \
    --target-dir "$TARGET_DIR"

echo "Building for Intel Simulator (x86_64-apple-ios)..."
cargo build $CARGO_FLAG \
    --target x86_64-apple-ios \
    --lib \
    --target-dir "$TARGET_DIR"

# --- Create a universal "fat" library for the simulator ---
echo "Creating universal library for simulators..."
SIMULATOR_UNIVERSAL_DIR="$TARGET_DIR/simulator-universal"
mkdir -p "$SIMULATOR_UNIVERSAL_DIR"

lipo -create \
  "$TARGET_DIR/aarch64-apple-ios-sim/$BUILD_TYPE/$BINARY_NAME" \
  "$TARGET_DIR/x86_64-apple-ios/$BUILD_TYPE/$BINARY_NAME" \
  -output "$SIMULATOR_UNIVERSAL_DIR/$BINARY_NAME"

# --- Create the XCFramework ---
echo "Creating $FRAMEWORK_NAME..."
rm -rf "target/$FRAMEWORK_NAME"

HEADERS_DIR_PLACEHOLDER="$TARGET_DIR/headers"
mkdir -p "$HEADERS_DIR_PLACEHOLDER"

xcodebuild -create-xcframework \
  -library "$TARGET_DIR/aarch64-apple-ios/$BUILD_TYPE/$BINARY_NAME" \
  -headers "$HEADERS_DIR_PLACEHOLDER" \
  -library "$SIMULATOR_UNIVERSAL_DIR/$BINARY_NAME" \
  -headers "$HEADERS_DIR_PLACEHOLDER" \
  -output "target/$FRAMEWORK_NAME"

echo "Successfully created target/$FRAMEWORK_NAME"

# --- Copy the XCFramework to your React Native project ---
DEST_XCFRAMEWORK_DIR="../../react-native-nitro-ark/react-native-nitro-ark/Ark.xcframework"
echo "Copying $FRAMEWORK_NAME to $DEST_XCFRAMEWORK_DIR"
rm -rf "$DEST_XCFRAMEWORK_DIR"
cp -R "target/$FRAMEWORK_NAME" "$DEST_XCFRAMEWORK_DIR"

DEST_XCFRAMEWORK_EXAMPLE_DIR="../../react-native-nitro-ark/react-native-nitro-ark/example/ios/Ark.xcframework"
echo "Copying $FRAMEWORK_NAME to $DEST_XCFRAMEWORK_EXAMPLE_DIR"
rm -rf "$DEST_XCFRAMEWORK_EXAMPLE_DIR"
cp -R "target/$FRAMEWORK_NAME" "$DEST_XCFRAMEWORK_EXAMPLE_DIR"

echo "Build complete!"
