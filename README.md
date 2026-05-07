# r0z

[中文说明](./README.zh-CN.md)

Standalone implementation of zygisk, providing zygisk API support for KernelSU and a replacement for Magisk's built-in zygisk.

## Current Runtime

+ Uses a native bridge loader (`libzn_loader.so`) to trigger early zygote loading
+ Uses `r0zd` as the companion daemon
+ Uses `libr0zgk.so` as the injected payload library
+ Ships a WebUI for status inspection

## Layout

+ `loader/`: native loader and injector implementation
+ `zygiskd/`: Rust daemon and root implementation adapters
+ `module/`: module packaging scripts, metadata and web UI assets
+ `loader/src/external/lsplt/`: vendored third-party dependency source

## Build

### Environment

+ JDK 17
+ Android SDK
+ Android NDK 26.1.10909125
+ Rust nightly
+ Rust targets:
    + `aarch64-linux-android`
    + `armv7-linux-androideabi`
    + `i686-linux-android`
    + `x86_64-linux-android`

### Commands

```bash
export JAVA_HOME=/path/to/jdk-17
rustup override set nightly
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
./gradlew zipRelease
```

Release artifacts will be generated under `module/build/outputs/release/`.

Current release zip naming:

```text
r0z-v1.0.3-4-release.zip
```

If `module/private_key` and `module/public_key` are absent, the module still builds but will not be signed.

### Notes

+ If Gradle reports that Android Gradle Plugin requires Java 11+ or Java 17, check `JAVA_HOME` first.
+ `local.properties` is intentionally not committed. Use your own local Android SDK configuration.
+ This repository vendors `lsplt` source directly. Cloning with `--recurse-submodules` is not required.

## Installation

### Install Prebuilt Package

1. Build a release package with `./gradlew zipRelease`, or download a prebuilt release artifact.
2. Install the generated zip from the KernelSU app or the Magisk app.
3. Reboot the device if your root manager does not reboot automatically after installation.

### Important

+ Recovery installation is not supported.
+ Android 8.0+ only (`minSdk 26`).
+ Supported ABIs: `armeabi-v7a`, `arm64-v8a`, `x86`, `x86_64`.
+ On Magisk, built-in zygisk must be disabled before installing r0z.
+ On KernelSU, make sure both the kernel-side KernelSU version and the Manager version meet the minimum requirements below.

## Requirements

### General

+ No multiple root implementation installed

### KernelSU

+ Minimal KernelSU version: 10940
+ Minimal KernelSU Manager (`ksud`) version: `11425`
+ Kernel has full SELinux patch support

### Magisk

+ Minimal version: 26402
+ Built-in zygisk turned off

## Compatibility

`PROCESS_ON_DENYLIST` cannot be flagged correctly for isolated processes on Magisk DenyList currently.

r0z only guarantees zygisk API compatibility. It does not guarantee Magisk internal private behaviors.

## Features

+ Native bridge based loader path with `libzn_loader.so`
+ Rust daemon `r0zd`
+ Injected payload library `libr0zgk.so`
+ WebUI for runtime status, module discovery and diagnostics

## FAQ

### Why does the build fail with a Java version error?

Your Gradle runtime is using the wrong JDK. Set `JAVA_HOME` to JDK 17 and retry.

### Why does installation fail from recovery?

Recovery installation is intentionally blocked. Install the module from the KernelSU app or the Magisk app instead.

### Why does installation fail on Magisk?

Check that your Magisk version is at least `26402` and that built-in zygisk is turned off before installing r0z.

### Why does installation fail on KernelSU?

Check the kernel-side KernelSU version, the KernelSU Manager (`ksud`) version, and whether your kernel supports the required SELinux patches.

## Acknowledgements

+ topjohnwu / Magisk, for the original r0z design and related references
+ KernelSU, for root environment integration support
+ LSPosed / LSPlt, for the native PLT hooking dependency used by this project
+ Dr-TSNG / ZygiskNext, for the directory layout reference and the first-version implementation reference

## Third-Party Dependencies

+ `LSPlt`
  Source: `loader/src/external/lsplt/`
  Upstream: <https://github.com/LSPosed/lsplt>
  License: see `loader/src/external/lsplt/LICENSE`
