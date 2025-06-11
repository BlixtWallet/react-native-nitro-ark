#!/bin/bash

# Exit on error
set -e

# --- Configuration ---
# IMPORTANT: Change this to the name of your crate as defined in your Cargo.toml
CRATE_NAME="bark-cpp" 
TARGET_DIR="target/ios"
BINARY_NAME="libbark_cpp.a"
FRAMEWORK_NAME="Ark.xcframework"

# --- Clean only the specific package artifacts ---
# This is much faster than `rm -rf` as it preserves dependency caches.
# We clean before each platform build to be safe.
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
cargo build --release \
    --target aarch64-apple-ios \
    --lib \
    --target-dir "$TARGET_DIR"

echo "Building for Apple Silicon Simulator (aarch64-apple-ios-sim)..."
cargo build --release \
    --target aarch64-apple-ios-sim \
    --lib \
    --target-dir "$TARGET_DIR"

echo "Building for Intel Simulator (x86_64-apple-ios)..."
cargo build --release \
    --target x86_64-apple-ios \
    --lib \
    --target-dir "$TARGET_DIR"

# --- Create a universal "fat" library for the simulator ---
echo "Creating universal library for simulators..."
SIMULATOR_UNIVERSAL_DIR="$TARGET_DIR/simulator-universal"
mkdir -p "$SIMULATOR_UNIVERSAL_DIR"

lipo -create \
  "$TARGET_DIR/aarch64-apple-ios-sim/release/$BINARY_NAME" \
  "$TARGET_DIR/x86_64-apple-ios/release/$BINARY_NAME" \
  -output "$SIMULATOR_UNIVERSAL_DIR/$BINARY_NAME"

# --- Create the XCFramework ---
echo "Creating $FRAMEWORK_NAME..."
rm -rf "target/$FRAMEWORK_NAME"

HEADERS_DIR_PLACEHOLDER="$TARGET_DIR/headers"
mkdir -p "$HEADERS_DIR_PLACEHOLDER"
# If you use cbindgen, you would copy your header here, e.g.:
# cbindgen --config cbindgen.toml --crate $CRATE_NAME --output $HEADERS_DIR_PLACEHOLDER/bark.h

xcodebuild -create-xcframework \
  -library "$TARGET_DIR/aarch64-apple-ios/release/$BINARY_NAME" \
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
