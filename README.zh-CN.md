# r0zygisk

[English](./README.md)

r0zygisk 是一个独立实现的 R0z，提供对 R0z API 的支持，可用于 KernelSU，并可替代 Magisk 内置的 R0z。

## 目录结构

+ `loader/`：native loader 和 injector 实现
+ `zygiskd/`：Rust 守护进程以及 root 实现适配层
+ `module/`：模块打包脚本、模块元数据和 Web UI 资源
+ `loader/src/external/lsplt/`：随仓库一起提交的第三方依赖源码

## 构建

### 环境要求

+ JDK 17
+ Android SDK
+ Android NDK `26.1.10909125`
+ Rust nightly
+ Rust targets：
  + `aarch64-linux-android`
  + `armv7-linux-androideabi`
  + `i686-linux-android`
  + `x86_64-linux-android`

### 构建命令

```bash
export JAVA_HOME=/path/to/jdk-17
rustup override set nightly
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
./gradlew zipRelease
```

构建产物会输出到 `module/build/outputs/release/`。

如果 `module/private_key` 和 `module/public_key` 不存在，模块仍然可以正常构建，但生成的是未签名包。

### 说明

+ 如果 Gradle 提示 Android Gradle Plugin 需要 Java 11+ 或 Java 17，请先检查 `JAVA_HOME`。
+ 仓库中不会提交 `local.properties`，请使用你自己的本地 Android SDK 配置。
+ 本仓库直接包含 `lsplt` 源码，不需要使用 `--recurse-submodules` 克隆。

## 安装方式

### 安装预编译包

1. 使用 `./gradlew zipRelease` 构建 release 包，或者直接下载预编译发布包。
2. 在 KernelSU App 或 Magisk App 中安装生成的 zip。
3. 如果 root 管理器在安装后没有自动重启，请手动重启设备。

### 重要说明

+ 不支持从 recovery 安装。
+ 在 Magisk 环境下，安装 r0zygisk 前必须先关闭内置 R0z。
+ 在 KernelSU 环境下，需要同时满足内核侧 KernelSU 版本和 Manager 版本要求。

## 使用要求

### 通用

+ 不要同时安装多个 root 实现

### KernelSU

+ 最低 KernelSU 版本：`10940`
+ 最低 KernelSU Manager（`ksud`）版本：`11424`
+ 内核需要具备完整的 SELinux patch 支持

### Magisk

+ 最低版本：`26402`
+ 需要关闭内置 R0z

## 兼容性

目前在 Magisk DenyList 下，隔离进程的 `PROCESS_ON_DENYLIST` 标记还不能被正确设置。

r0zygisk 只保证 R0z API 层面的行为兼容，不保证 Magisk 内部私有功能也保持一致。

## 常见问题

### 为什么构建时报 Java 版本错误？

通常是因为 Gradle 实际使用的 JDK 不对。请将 `JAVA_HOME` 设置为 JDK 17 后再重试。

### 为什么不能从 recovery 安装？

项目安装脚本会直接拒绝 recovery 安装。请改为从 KernelSU App 或 Magisk App 中安装。

### 为什么生成的安装包是未签名的？

如果 `module/private_key` 和 `module/public_key` 不存在，构建仍然会成功，但输出包会按设计保持未签名状态。

### 为什么在 Magisk 上安装失败？

请确认 Magisk 版本不低于 `26402`，并且在安装前已经关闭内置 R0z。

### 为什么在 KernelSU 上安装失败？

请确认内核侧 KernelSU 版本、KernelSU Manager（`ksud`）版本，以及内核对所需 SELinux patch 的支持情况都满足要求。

## 致谢

+ topjohnwu / Magisk：提供了最初的 R0z 设计和相关参考实现
+ KernelSU：提供了对应的 root 环境集成基础
+ LSPosed / LSPlt：提供了本项目使用的 native PLT hook 依赖

## 第三方依赖说明

+ `LSPlt`
  源码位置：`loader/src/external/lsplt/`
  上游项目：<https://github.com/LSPosed/lsplt>
  许可证：见 `loader/src/external/lsplt/LICENSE`
