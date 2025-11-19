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

# --- Always build host target to refresh generated headers ---
echo "Building host target to refresh FFI headers..."
cargo build $CARGO_FLAG --lib
HOST_BUILD_DIR="target/$BUILD_TYPE/build"

# Prepare shared headers directory used later
HEADERS_DIR_CXX="$TARGET_DIR/cxx_headers"
rm -rf "$HEADERS_DIR_CXX"
mkdir -p "$HEADERS_DIR_CXX"

HOST_HEADER_SRC_PATH=$(find "$HOST_BUILD_DIR" -name "cxx.rs.h" | head -n 1)
if [ -z "$HOST_HEADER_SRC_PATH" ]; then
    echo "Error: Could not find host-generated cxx.rs.h header."
    exit 1
fi
echo "Copying host API header from: $HOST_HEADER_SRC_PATH"
cp "$HOST_HEADER_SRC_PATH" "$HEADERS_DIR_CXX/ark_cxx.h"

HOST_CXX_HEADER_PATH=$(find "$HOST_BUILD_DIR" -path "*/include/rust/cxx.h" | head -n 1)
if [ -z "$HOST_CXX_HEADER_PATH" ]; then
    echo "Error: Could not find host-generated cxx.h header."
    exit 1
fi
echo "Copying host cxx header from: $HOST_CXX_HEADER_PATH"
cp "$HOST_CXX_HEADER_PATH" "$HEADERS_DIR_CXX/cxx.h"

# --- Install Rust targets ---
echo "Ensuring required Rust targets are installed..."
rustup target add \
    aarch64-apple-ios \
    aarch64-apple-ios-sim

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

# --- Create the XCFramework ---
echo "Creating $FRAMEWORK_NAME..."
rm -rf "target/$FRAMEWORK_NAME"
rm -rf "target/$CXX_FRAMEWORK_NAME"

HEADERS_DIR_PLACEHOLDER="$TARGET_DIR/headers"
mkdir -p "$HEADERS_DIR_PLACEHOLDER"

xcodebuild -create-xcframework \
  -library "$TARGET_DIR/aarch64-apple-ios/$BUILD_TYPE/$BINARY_NAME" \
  -headers "$HEADERS_DIR_PLACEHOLDER" \
  -library "$TARGET_DIR/aarch64-apple-ios-sim/$BUILD_TYPE/$BINARY_NAME" \
  -headers "$HEADERS_DIR_PLACEHOLDER" \
  -output "target/$FRAMEWORK_NAME"

echo "Successfully created target/$FRAMEWORK_NAME"

echo "Creating $CXX_FRAMEWORK_NAME..."

# HEADERS_DIR_CXX already populated from the host build.
# Now HEADERS_DIR_CXX is the single source of truth for our headers.
# Use it to populate the cpp/generated directory for local builds.
DEST_HEADER_DIR="../react-native-nitro-ark/cpp/generated"
rm -rf "$DEST_HEADER_DIR"
mkdir -p "$DEST_HEADER_DIR"
cp "$HEADERS_DIR_CXX/ark_cxx.h" "$DEST_HEADER_DIR/"
cp "$HEADERS_DIR_CXX/cxx.h" "$DEST_HEADER_DIR/"

# Find the CXX bridge library for the device arch
DEVICE_CXX_LIB_PATH=$(find "$TARGET_DIR/aarch64-apple-ios/$BUILD_TYPE/build" -name "$CXX_BINARY_NAME" | head -n 1)
if [ -z "$DEVICE_CXX_LIB_PATH" ]; then
    echo "Error: Could not find CXX bridge library for device architecture."
    exit 1
fi

# Find the CXX bridge library for the simulator arch
SIM_ARM64_CXX_LIB_PATH=$(find "$TARGET_DIR/aarch64-apple-ios-sim/$BUILD_TYPE/build" -name "$CXX_BINARY_NAME" | head -n 1)
if [ -z "$SIM_ARM64_CXX_LIB_PATH" ]; then
    echo "Error: Could not find CXX bridge library for simulator architecture."
    exit 1
fi

xcodebuild -create-xcframework \
    -library "$DEVICE_CXX_LIB_PATH" \
    -headers "$HEADERS_DIR_CXX" \
    -library "$SIM_ARM64_CXX_LIB_PATH" \
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
