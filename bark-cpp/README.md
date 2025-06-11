# Bark C++ FFI

This directory contains the C++ Foreign Function Interface (FFI) code required to communicate with the underlying Rust "Ark" project.

## Purpose

The primary goal of this code is to provide a C++-callable API that bridges the gap between the native mobile environment (iOS/Android) and the Rust core logic within the "Ark" project. This allows higher-level applications, such as a React Native application, to interact with the Rust functionalities through a C++ layer.

## How it Works

The C++ code in this directory typically involves:

1.  **Defining C-compatible function signatures:** These functions will be exposed from the Rust side (using `#[no_mangle]` and `extern "C"`) and called from this C++ code.
2.  **Handling data marshalling:** Converting data types between C++ and Rust (e.g., strings, numbers, complex objects). This might involve serialization/deserialization or direct memory manipulation.
3.  **Exposing a C++ API:** Providing a clean C++ interface that encapsulates the FFI calls, making it easier for other C++ code (like the React Native Nitro module) to consume.

## Building

The code in this directory is compiled into a static library (e.g., `.a` for iOS, `.so` for Android). This static library is then linked into the main mobile application, allowing the React Native Nitro module to call its functions.

Refer to the main project's build system for specific compilation instructions for different platforms.

## Relationship to Other Components

-   **Ark (Rust Project):** This C++ code directly calls functions exposed by the Ark Rust project.
-   **React Native Nitro Module ([`react-native-nitro-ark/cpp`](../react-native-nitro-ark/cpp))**: The C++ code in the React Native module utilizes the API provided by this [`bark-cpp`](./) layer to execute Rust logic.

## Setting up the project

### Build using nix on a Mac
- Install [nix](https://determinate.systems/nix-installer/)
- Install [direnv](https://direnv.net/)
- Run `direnv allow` to allow direnv to load the nix environment.
- For iOS you will need to install XCode.
- For Android you will need to install Android Studio.

## Building binaries

- Android - Run the `build-android.sh` script to build the Android binary.
- iOS - Run the `build-ios.sh` script to build the iOS binary.
- Mac - You can also run the main.rs file locally using `cargo run --bin bark`.
