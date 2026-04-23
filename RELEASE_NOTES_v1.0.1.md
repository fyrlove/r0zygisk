# r0z v1.0.1 (versionCode 2)

## Highlights
- Rename and identity unification: `r0zygisk` -> `r0z`.
- Runtime architecture switched to native bridge loader flow.
- Zygisk payload library renamed: `libzygisk.so` -> `libr0zgk.so`.
- Daemon renamed: `zygiskd` -> `r0zd`.
- Control script renamed: `zygisk-ctl.sh` -> `r0z-ctl.sh`.

## Runtime / Injection Changes
- Added native bridge components:
  - `libzn_loader.so`
  - `libpayload.so`
- `post-fs-data.sh` and `service.sh` now:
  - start `r0zd` in daemon mode,
  - set `ro.dalvik.vm.native.bridge=libzn_loader.so` (when `resetprop` is available),
  - wait for daemon socket readiness to reduce zygote race conditions.

## Packaging Changes
- Installation library paths migrated to:
  - `$MODPATH/system/lib`
  - `$MODPATH/system/lib64`
- Packaging/signature mappings updated for:
  - `libr0zgk.so`
  - `libzn_loader.so`
  - `libpayload.so`
- Release package naming for this version:
  - `r0z-v1.0.1-2-<buildType>.zip`

## UI / Observability
- WebUI updated from tracer-control centric workflow to status-observation workflow.
- Added native bridge status visibility in WebUI diagnostics.
- Updated module discovery and display naming to `r0z`.

## Toolchain / Project
- Gradle project/module identifiers and namespace aligned to `r0z`.
- Rust crate and Gradle module task paths aligned to `r0zd`.

## Compatibility Notes
- Existing minimum KernelSU / Magisk requirements remain unchanged.
- If `module/private_key` and `module/public_key` are missing, builds remain unsigned by design.

## Upgrade Notes
- Recommended: reboot once after upgrading from `v1.0.0` to ensure native bridge and daemon status fully settle.

## Full Diff Document
- See `R0ZYGISK_V2_DIFF_FROM_V1.md` for the complete v2-v1 change log and file-level diff scope.
