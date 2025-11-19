#!/bin/bash

# Exit on error
set -e

# Unset any iOS/macOS specific variables that might interfere
unset SDKROOT
unset PLATFORM_NAME
unset IPHONEOS_DEPLOYMENT_TARGET
unset TVOS_DEPLOYMENT_TARGET
unset XROS_DEPLOYMENT_TARGET

# Verify ANDROID_HOME is set
if [ -z "$ANDROID_HOME" ]; then
    echo "Error: ANDROID_HOME is not set"
    echo "Make sure you're in the Nix shell"
    exit 1
fi

# Set NDK paths
NDK_VERSION="26.1.10909125"
NDK_PATH="$ANDROID_HOME/ndk/$NDK_VERSION"

if [ ! -d "$NDK_PATH" ]; then
    echo "Error: Could not find Android NDK at $NDK_PATH"
    exit 1
fi

echo "Found NDK at: $NDK_PATH"

# Set Android API level
API_LEVEL=30

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

# Directory for generated headers consumed by RN project
DEST_HEADER_DIR="../react-native-nitro-ark/cpp/generated"

# Always refresh host-generated headers before cross-compiling
echo "Building host target to refresh FFI headers..."
cargo build $CARGO_FLAG --lib
HOST_BUILD_DIR="target/$BUILD_TYPE/build"

HOST_HEADER_SRC_PATH=$(find "$HOST_BUILD_DIR" -name "cxx.rs.h" | head -n 1)
if [ -z "$HOST_HEADER_SRC_PATH" ]; then
    echo "Error: Could not find host-generated cxx.rs.h header."
    exit 1
fi
echo "Copying host API header from: $HOST_HEADER_SRC_PATH"
mkdir -p "$DEST_HEADER_DIR"
cp -f "$HOST_HEADER_SRC_PATH" "$DEST_HEADER_DIR/ark_cxx.h"

HOST_CXX_HEADER_PATH=$(find "$HOST_BUILD_DIR" -path "*/include/rust/cxx.h" | head -n 1)
if [ -z "$HOST_CXX_HEADER_PATH" ]; then
    echo "Error: Could not find host-generated cxx.h header."
    exit 1
fi
echo "Copying host cxx header from: $HOST_CXX_HEADER_PATH"
cp -f "$HOST_CXX_HEADER_PATH" "$DEST_HEADER_DIR/cxx.h"

# Define build variables
OUTPUT_DIR="target/jniLibs"
BINARY_NAME="libbark_cpp.a"
CXX_BINARY_NAME="libcxxbridge1.a"

# Delete old output directory
rm -rf "$OUTPUT_DIR"

# Create output directory structure
mkdir -p "$OUTPUT_DIR/arm64-v8a"
mkdir -p "$OUTPUT_DIR/x86_64"

echo "Building for Android..."

# Determine host platform prefix
HOST_TAG="linux-x86_64"
if [[ "$(uname)" == "Darwin" ]]; then
    HOST_TAG="darwin-x86_64"
fi

TOOLCHAIN_PATH="$NDK_PATH/toolchains/llvm/prebuilt/$HOST_TAG"

if [ ! -d "$TOOLCHAIN_PATH" ]; then
    echo "Error: Could not find NDK toolchain at $TOOLCHAIN_PATH"
    exit 1
fi

# Set up common environment variables for the toolchain
export PATH="$TOOLCHAIN_PATH/bin:$PATH"
export RANLIB="$TOOLCHAIN_PATH/bin/llvm-ranlib"
export AR="$TOOLCHAIN_PATH/bin/llvm-ar"
export AS="$TOOLCHAIN_PATH/bin/llvm-as"
export NM="$TOOLCHAIN_PATH/bin/llvm-nm"
export STRIP="$TOOLCHAIN_PATH/bin/llvm-strip"

# --- Build for ARM64 (aarch64-linux-android) ---
echo "Building for arm64-v8a..."
TARGET_ARCH_ARM64="aarch64-linux-android"
TARGET_DIR_ARM64="target/$TARGET_ARCH_ARM64/$BUILD_TYPE"

export TARGET_AR="$TOOLCHAIN_PATH/bin/llvm-ar"
export TARGET_CC="$TOOLCHAIN_PATH/bin/aarch64-linux-android$API_LEVEL-clang"
export TARGET_CXX="$TOOLCHAIN_PATH/bin/aarch64-linux-android$API_LEVEL-clang++"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_AR="$TOOLCHAIN_PATH/bin/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$TOOLCHAIN_PATH/bin/aarch64-linux-android$API_LEVEL-clang"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_RANLIB="$TOOLCHAIN_PATH/bin/llvm-ranlib"
export OPENSSL_INCLUDE_DIR="$PWD/target/$TARGET_ARCH_ARM64/$BUILD_TYPE/build/openssl-sys-*/out/include"
export OPENSSL_LIB_DIR="$PWD/target/$TARGET_ARCH_ARM64/$BUILD_TYPE/build/openssl-sys-*/out/lib"

rustup target add $TARGET_ARCH_ARM64
cargo build --target=$TARGET_ARCH_ARM64 $CARGO_FLAG --lib
cp "$TARGET_DIR_ARM64/$BINARY_NAME" "$OUTPUT_DIR/arm64-v8a/"
ARM64_CXX_LIB_PATH=$(find "$TARGET_DIR_ARM64/build" -name "$CXX_BINARY_NAME" | head -n 1)
if [ -z "$ARM64_CXX_LIB_PATH" ]; then
    echo "Error: Could not find CXX bridge library for arm64-v8a."
    exit 1
fi
cp "$ARM64_CXX_LIB_PATH" "$OUTPUT_DIR/arm64-v8a/"

# --- Build for x86_64 (x86_64-linux-android) ---
echo "Building for x86_64..."
TARGET_ARCH_X86_64="x86_64-linux-android"
TARGET_DIR_X86_64="target/$TARGET_ARCH_X86_64/$BUILD_TYPE"

export TARGET_AR="$TOOLCHAIN_PATH/bin/llvm-ar"
export TARGET_CC="$TOOLCHAIN_PATH/bin/x86_64-linux-android$API_LEVEL-clang"
export TARGET_CXX="$TOOLCHAIN_PATH/bin/x86_64-linux-android$API_LEVEL-clang++"
export CARGO_TARGET_X86_64_LINUX_ANDROID_AR="$TOOLCHAIN_PATH/bin/llvm-ar"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$TOOLCHAIN_PATH/bin/x86_64-linux-android$API_LEVEL-clang"
export CARGO_TARGET_X86_64_LINUX_ANDROID_RANLIB="$TOOLCHAIN_PATH/bin/llvm-ranlib"
export OPENSSL_INCLUDE_DIR="$PWD/target/$TARGET_ARCH_X86_64/$BUILD_TYPE/build/openssl-sys-*/out/include"
export OPENSSL_LIB_DIR="$PWD/target/$TARGET_ARCH_X86_64/$BUILD_TYPE/build/openssl-sys-*/out/lib"

rustup target add $TARGET_ARCH_X86_64
cargo build --target=$TARGET_ARCH_X86_64 $CARGO_FLAG --lib
cp "$TARGET_DIR_X86_64/$BINARY_NAME" "$OUTPUT_DIR/x86_64/"
X86_64_CXX_LIB_PATH=$(find "$TARGET_DIR_X86_64/build" -name "$CXX_BINARY_NAME" | head -n 1)
if [ -z "$X86_64_CXX_LIB_PATH" ]; then
    echo "Error: Could not find CXX bridge library for x86_64."
    exit 1
fi
cp "$X86_64_CXX_LIB_PATH" "$OUTPUT_DIR/x86_64/"

# --- Copy binaries to React Native project ---
DEST_JNI_DIR_ARM64="../../react-native-nitro-ark/react-native-nitro-ark/android/src/main/jniLibs/arm64-v8a"
DEST_JNI_DIR_X86_64="../../react-native-nitro-ark/react-native-nitro-ark/android/src/main/jniLibs/x86_64"

mkdir -p "$DEST_JNI_DIR_ARM64"
mkdir -p "$DEST_JNI_DIR_X86_64"

echo "Copying arm64-v8a binary..."
cp -f "$OUTPUT_DIR/arm64-v8a/$BINARY_NAME" "$DEST_JNI_DIR_ARM64/"
cp -f "$OUTPUT_DIR/arm64-v8a/$CXX_BINARY_NAME" "$DEST_JNI_DIR_ARM64/"

echo "Copying x86_64 binary..."
cp -f "$OUTPUT_DIR/x86_64/$BINARY_NAME" "$DEST_JNI_DIR_X86_64/"
cp -f "$OUTPUT_DIR/x86_64/$CXX_BINARY_NAME" "$DEST_JNI_DIR_X86_64/"

echo "Android build complete!"
