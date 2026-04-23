# r0zygisk

[中文说明](./README.zh-CN.md)

Standalone implementation of R0z, providing R0z API support for KernelSU and a replacement of Magisk's built-in R0z.

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
+ On Magisk, built-in R0z must be disabled before installing r0zygisk.
+ On KernelSU, make sure both the kernel-side KernelSU version and the Manager version meet the minimum requirements below.

## Requirements

### General

+ No multiple root implementation installed

### KernelSU

+ Minimal KernelSU version: 10940
+ Minimal KernelSU Manager (ksud) version: 11424
+ Kernel has full SELinux patch support

### Magisk

+ Minimal version: 26402
+ Built-in R0z turned off

## Compatibility

`PROCESS_ON_DENYLIST` cannot be flagged correctly for isolated processes on Magisk DenyList currently.

r0zygisk only guarantees the same behavior of R0z API, but will NOT ensure Magisk's internal features.

## FAQ

### Why does the build fail with a Java version error?

Your Gradle runtime is using the wrong JDK. Set `JAVA_HOME` to JDK 17 and retry.

### Why does installation fail from recovery?

Recovery installation is intentionally blocked. Install the module from the KernelSU app or the Magisk app instead.

### Why is the generated package unsigned?

If `module/private_key` and `module/public_key` are not present, the build still succeeds, but the output package is unsigned by design.

### Why does installation fail on Magisk?

Check that your Magisk version is at least `26402` and that built-in R0z is turned off before installing r0zygisk.

### Why does installation fail on KernelSU?

Check the kernel-side KernelSU version, the KernelSU Manager (`ksud`) version, and whether your kernel supports the required SELinux patches.

## Acknowledgements

+ topjohnwu / Magisk, for the original R0z design and related references
+ KernelSU, for root environment integration support
+ LSPosed / LSPlt, for the native PLT hooking dependency used by this project

## Third-Party Dependencies

+ `LSPlt`
  Source: `loader/src/external/lsplt/`
  Upstream: <https://github.com/LSPosed/lsplt>
  License: see `loader/src/external/lsplt/LICENSE`
