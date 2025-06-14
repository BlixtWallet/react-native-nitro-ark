# React Native Nitro Ark Module

This directory contains the React Native module that bridges your React Native application with the underlying Rust [Ark](https://codeberg.org/ark-bitcoin/bark) project by [Second](https://second.tech), facilitated by the C++ FFI layer in [`bark-cpp`](./bark-cpp). It leverages [React Native NitroModules](https://github.com/mrousavy/nitro) for efficient communication between JavaScript/TypeScript and native C++ code.

## Installing the react-native-nitro-ark module

- To install the `react-native-nitro-ark` module, run the following command in your React Native project directory:
- For all methods and type definitions refer to the [`react-native-nitro-ark/src/index.tsx`](./react-native-nitro-ark/src/index.tsx) file.

```bash
npm install react-native-nitro-ark react-native-nitro-modules
# or
yarn add react-native-nitro-ark react-native-nitro-modules
```

- Download the Android binary and put it inside `node_modules/react-native-nitro-ark/android/src/main/jniLibs/arm64-v8a`.
- Download the iOS binary, `unzip` it and put it inside `node_modules/react-native-nitro-ark/Ark.xcframework`.

## Purpose

The primary goal of this module is to expose the rust functions of the "Ark" Rust project to your React Native application. It provides:

1.  A TypeScript API ([`src/`](./react-native-nitro-ark/src/)) for easy consumption from your React Native JavaScript/TypeScript code.
2.  Native C++ implementations ([`cpp/`](./react-native-nitro-ark/cpp/)) that utilize the [`bark-cpp`](./bark-cpp/) FFI to call into the Rust core logic.

This allows you to write high-performance core logic in Rust and seamlessly integrate it into your cross-platform React Native application.

## Directory Structure

-   **[`bark-cpp/`](./bark-cpp/)**: Contains the C++ FFI code that directly interfaces with the Rust "Ark" project. This code is compiled into a static library for Android and iOS.
-   **[`react-native-nitro-ark/cpp/`](./react-native-nitro-ark/cpp/)**: Contains the C++ code specific to this React Native Nitro module. This code:
    -   Includes the necessary headers from React Native Nitro.
    -   Links against the static library produced by `bark-cpp`.
    -   Implements the native methods that are exposed to the TypeScript side.
    -   Calls the functions provided by the `bark-cpp` FFI layer.
-   **[`react-native-nitro-ark/react-native/src/`](./react-native-nitro-ark/react-native/src/)**: Contains the TypeScript/JavaScript code that defines the public API of this module. This is what your React Native application will import and use. It makes calls to the native C++ methods defined in the [`cpp/`](./react-native-nitro-ark/cpp/) directory via the React Native Nitro bridge.
-   **[`react-native-nitro-ark/example/`](./react-native-nitro-ark/example/)**: Contains an example React Native application that demonstrates how to use the `react-native-nitro-ark` module. This directory includes the necessary files to set up a React Native project and showcases the usage of the TypeScript API.

## How it Works

The interaction flow is generally as follows:

1.  **React Native App (JS/TS):** Your application code imports and calls functions from the TypeScript API exposed in [`react-native-nitro-ark/src/`](./react-native-nitro-ark/src/).
2.  **TypeScript API ([`react-native-nitro-ark/src/`](./react-native-nitro-ark/src/))**: These TypeScript functions act as a wrapper. They use React Native Nitro's mechanisms to invoke corresponding native C++ methods.
3.  **React Native Nitro Bridge:** Nitro efficiently marshals data and forwards the call from JavaScript to the native C++ environment.
4.  **Nitro C++ Module ([`react-native-nitro-ark/cpp/`](./react-native-nitro-ark/cpp/))**: The C++ methods implemented here receive the call.
5.  **FFI Call:** This C++ code then calls the relevant functions exposed by the [`bark-cpp`](./bark-cpp/) FFI layer. These [`bark-cpp`](./bark-cpp/) functions are available because the static library produced from [`bark-cpp`](./bark-cpp/) is linked into the application.
6.  **Bark C++ FFI ([`react-native-nitro-ark/bark-cpp/`](./react-native-nitro-ark/bark-cpp/))**: This layer translates the C++ call into a call to the Rust "Ark" project's compiled code.

## Building and Integration

-   The `bark-cpp` code needs to be compiled into static libraries for each target platform (iOS and Android).
-   This React Native module (`react-native-nitro-ark`) then links against these precompiled static libraries.
-   Ensure that your main application's build system (Xcode for iOS, Gradle for Android) is configured to:
    -   Compile and link the C++ code in [`react-native-nitro-ark/cpp/`](./react-native-nitro-ark/cpp/).
    -   Link the static libraries from `bark-cpp`.
    -   Integrate the React Native Nitro module correctly.

Refer to the React Native Nitro documentation and the specific build configurations within the [`ios/`](./react-native-nitro-ark/ios/) and [`android/`](./react-native-nitro-ark/android/) directories for detailed integration steps.

## Dependencies

-   Rust
-   React Native
-   The compiled static libraries from `bark-cpp`.
