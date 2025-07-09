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
CXX_BINARY_NAME="libcxxbridge1.a"
FRAMEWORK_NAME="Ark.xcframework"
CXX_FRAMEWORK_NAME="ArkCxxBridge.xcframework"

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

echo "Creating universal library for CXX bridge simulators..."
SIMULATOR_CXX_UNIVERSAL_DIR="$TARGET_DIR/simulator-cxx-universal"
mkdir -p "$SIMULATOR_CXX_UNIVERSAL_DIR"

# Find the CXX bridge library for each simulator arch
SIM_ARM64_CXX_LIB_PATH=$(find "$TARGET_DIR/aarch64-apple-ios-sim/$BUILD_TYPE/build" -name "$CXX_BINARY_NAME" | head -n 1)
SIM_X86_64_CXX_LIB_PATH=$(find "$TARGET_DIR/x86_64-apple-ios/$BUILD_TYPE/build" -name "$CXX_BINARY_NAME" | head -n 1)

if [ -z "$SIM_ARM64_CXX_LIB_PATH" ] || [ -z "$SIM_X86_64_CXX_LIB_PATH" ]; then
    echo "Error: Could not find CXX bridge library for one or more simulator architectures."
    exit 1
fi

lipo -create \
  "$SIM_ARM64_CXX_LIB_PATH" \
  "$SIM_X86_64_CXX_LIB_PATH" \
  -output "$SIMULATOR_CXX_UNIVERSAL_DIR/$CXX_BINARY_NAME"

# --- Create the XCFramework ---
echo "Creating $FRAMEWORK_NAME..."
rm -rf "target/$FRAMEWORK_NAME"
rm -rf "target/$CXX_FRAMEWORK_NAME"

HEADERS_DIR_PLACEHOLDER="$TARGET_DIR/headers"
mkdir -p "$HEADERS_DIR_PLACEHOLDER"

xcodebuild -create-xcframework \
  -library "$TARGET_DIR/aarch64-apple-ios/$BUILD_TYPE/$BINARY_NAME" \
  -headers "$HEADERS_DIR_PLACEHOLDER" \
  -library "$SIMULATOR_UNIVERSAL_DIR/$BINARY_NAME" \
  -headers "$HEADERS_DIR_PLACEHOLDER" \
  -output "target/$FRAMEWORK_NAME"

echo "Successfully created target/$FRAMEWORK_NAME"

echo "Creating $CXX_FRAMEWORK_NAME..."
HEADERS_DIR_CXX="$TARGET_DIR/cxx_headers"
mkdir -p "$HEADERS_DIR_CXX"
HEADER_SRC_PATH=$(find "$TARGET_DIR/aarch64-apple-ios/$BUILD_TYPE/build" -name "cxx.rs.h" | head -n 1)
if [ -z "$HEADER_SRC_PATH" ]; then
    echo "Error: Could not find generated cxx.rs.h header."
    exit 1
fi
echo "Found cxx header at: $HEADER_SRC_PATH"
cp "$HEADER_SRC_PATH" "$HEADERS_DIR_CXX/ark_cxx.h"

# Also copy to the react-native project for direct include
DEST_HEADER_DIR="../react-native-nitro-ark/cpp/generated"
mkdir -p "$DEST_HEADER_DIR"
cp "$HEADER_SRC_PATH" "$DEST_HEADER_DIR/ark_cxx.h"

# Find the CXX bridge library for the device arch
DEVICE_CXX_LIB_PATH=$(find "$TARGET_DIR/aarch64-apple-ios/$BUILD_TYPE/build" -name "$CXX_BINARY_NAME" | head -n 1)
if [ -z "$DEVICE_CXX_LIB_PATH" ]; then
    echo "Error: Could not find CXX bridge library for device architecture."
    exit 1
fi

xcodebuild -create-xcframework \
    -library "$DEVICE_CXX_LIB_PATH" \
    -headers "$HEADERS_DIR_CXX" \
    -library "$SIMULATOR_CXX_UNIVERSAL_DIR/$CXX_BINARY_NAME" \
    -headers "$HEADERS_DIR_CXX" \
    -output "target/$CXX_FRAMEWORK_NAME"

echo "Successfully created target/$CXX_FRAMEWORK_NAME"

# --- Copy the XCFramework to your React Native project ---
DEST_XCFRAMEWORK_DIR="../../react-native-nitro-ark/react-native-nitro-ark"
echo "Copying frameworks to $DEST_XCFRAMEWORK_DIR"
rm -rf "$DEST_XCFRAMEWORK_DIR/$FRAMEWORK_NAME"
rm -rf "$DEST_XCFRAMEWORK_DIR/$CXX_FRAMEWORK_NAME"
cp -R "target/$FRAMEWORK_NAME" "$DEST_XCFRAMEWORK_DIR/"
cp -R "target/$CXX_FRAMEWORK_NAME" "$DEST_XCFRAMEWORK_DIR/"

DEST_XCFRAMEWORK_EXAMPLE_DIR="../../react-native-nitro-ark/react-native-nitro-ark/example/ios"
echo "Copying frameworks to $DEST_XCFRAMEWORK_EXAMPLE_DIR"
rm -rf "$DEST_XCFRAMEWORK_EXAMPLE_DIR/$FRAMEWORK_NAME"
rm -rf "$DEST_XCFRAMEWORK_EXAMPLE_DIR/$CXX_FRAMEWORK_NAME"
cp -R "target/$FRAMEWORK_NAME" "$DEST_XCFRAMEWORK_EXAMPLE_DIR/"
cp -R "target/$CXX_FRAMEWORK_NAME" "$DEST_XCFRAMEWORK_EXAMPLE_DIR/"

echo "Build complete!"
