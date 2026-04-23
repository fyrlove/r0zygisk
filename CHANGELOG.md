# Changelog

All notable changes to this project will be documented in this file.

## v1.0.2

### Changed

+ Bumped module version to `v1.0.2` and `versionCode` to `3`
+ Fixed GitHub Actions release artifact patterns from `r0zygisk-v*` to `r0z-v*`
+ Updated CI release artifact extraction/output directories to match `r0z` naming

### Notes

+ This release primarily ensures tag-triggered GitHub Releases can publish built artifacts correctly
+ Release package naming follows `r0z-v1.0.2-3-<buildType>.zip`

## v1.0.1

### Added

+ Introduced a new zygisk payload library name: `libr0zgk.so`
+ Added native bridge loader integration that explicitly loads `libr0zgk.so` from `/system/lib(64)`
+ Added package-level consistency checks/sign entries for `libr0zgk.so` in module assembly

### Changed

+ Bumped module version to `v1.0.1` and `versionCode` to `2`
+ Updated installer extraction paths and packaging mappings to use `libr0zgk.so` instead of `libzygisk.so`
+ Updated Web UI status description text to reflect the new runtime loader path and library naming

### Notes

+ r0zygisk 第二版
+ Release package naming now follows `r0z-v1.0.1-2-<buildType>.zip`
+ Existing unsigned build behavior remains unchanged when `module/private_key` and `module/public_key` are absent

## v1.0.0

### Added

+ Initial public open source release of `r0zygisk`
+ English and Simplified Chinese README documents
+ Build instructions, installation guide, FAQ, acknowledgements and third-party dependency notes
+ Vendored `lsplt` source in the repository for direct builds without submodules
+ GitHub Actions CI workflow for automated build and artifact upload
+ Automatic GitHub Release publishing for version tags

### Changed

+ Clean repository layout for public release
+ Remove local build artifacts, machine-specific files and embedded git metadata from vendored source
+ Make the project buildable in a clean environment with JDK 17, Android SDK/NDK and Rust nightly
+ Align CI with the current repository layout and `main` branch
+ Keep release/debug package naming consistent with `r0zygisk`

### Notes

+ r0zygisk 第一版
+ Release packages can be built with `./gradlew zipRelease`
+ If `module/private_key` and `module/public_key` are absent, generated packages are unsigned by design
