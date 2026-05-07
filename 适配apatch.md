# 适配 APatch 记录

## 背景

- 设备从 `Sukisu` 切换到 `APatch 11142`
- 使用 `zygisk-gadget-420.zip` 后，勾选目标 app 启动不阻塞
- 目标是确认原因、给当前 `r0z` 增加 `APatch` 兼容、保留现有 `KernelSU/Magisk/Sukisu` 路径，并完成打包发布

## 核心分析结论

### `zygisk-gadget` 的预期行为

- 包内 `libgadget.config.so` 配置为：
  - `interaction.type = listen`
  - `on_load = wait`
  - `port = 14725`
- 只要 gadget 真正注入目标进程，目标应用就应该阻塞，直到调试端连接

### 原始 `r0z` 的问题

- 后端 root 实现原本只支持：
  - `KernelSU`
  - `Magisk`
- 没有：
  - `APatch` root impl
  - `APatch` 权限判断
  - `APatch` manager 识别
  - `APatch` process flags
  - `APatch` 安装分支

### APatch 11142 的关键特征

- APatch 本身没有内建 Zygisk，需要依赖外部 APM 提供 Zygisk 环境
- APatch root/排除数据来自：
  - `/data/adb/ap/package_config`
- APatch manager 默认包名：
  - `me.bmax.apatch`
- APatch 工作目录：
  - `/data/adb/ap/`
  - `/data/adb/apd`

## 本次代码修改

### 新增

- `zygiskd/src/root_impl/apatch.rs`
  - `get_apatch()`
  - `uid_granted_root(uid)`
  - `uid_should_umount(uid)`
  - `uid_is_manager(uid)`

### 调整

- `zygiskd/src/root_impl/mod.rs`
  - 新增 `RootImpl::APatch`
  - `setup()` 同时检测 `KernelSU / Magisk / APatch`

- `zygiskd/src/constants.rs`
  - 新增 `PROCESS_ROOT_IS_APATCH`

- `zygiskd/src/r0zd.rs`
  - APatch 进入 daemon info
  - APatch 进入 process flags

- `loader/src/injector/module.hpp`
  - 新增 `PROCESS_ROOT_IS_APATCH`

- `loader/src/injector/hook.cpp`
  - APatch 走专属 root flag
  - manager 特判扩展到 APatch

- `loader/src/injector/unmount.cpp`
  - 新增 `revert_unmount_apatch()`
  - 第一版复用 Magisk 风格 `/data/adb` 清理逻辑

- `loader/src/injector/zygisk.hpp`
  - 补 `revert_unmount_apatch()` 声明

- `module/src/customize.sh`
  - 新增 APatch 安装分支

- `module/src/webroot/index.html`
  - Root 文案加入 APatch

- `module/src/webroot/app.js`
  - 模块探测兼容 APatch 相关目录

## ADB 实测结论

### `r0z` 主链路已打通

执行：

```bash
adb shell su -c '/data/adb/modules/r0z/bin/r0zd root'
```

输出：

```text
root impl: APatch
```

执行：

```bash
adb shell su -c 'cat /data/adb/modules/r0z/status.json'
```

结果显示：

- `zygote64 = injected`
- `daemon64 = running`
- `daemon64_info = Root: APatch,module_count: 2`

执行：

```bash
adb shell getprop ro.dalvik.vm.native.bridge
```

输出：

```text
libzn_loader.so
```

结论：

- `r0z` 已经兼容并运行在 `APatch`
- native bridge、daemon、zygote 注入链路都正常

### 当前 `zygisk-gadget` 不阻塞的直接原因

执行：

```bash
adb shell su -c 'cat /data/data/com.xiaojia.xgj/files/config.json'
```

实测结果显示：

- `com.ss.android.article.news.inject = false`
- `me.bmax.apatch.inject = true`

执行：

```bash
adb shell su -c 'ss -ltnp | grep 14725 || true'
```

没有监听结果。

另外实测：

```bash
adb shell su -c 'cat /data/adb/ap/package_config'
```

对应文件是空文件。

结论：

- 当前头条没有真正启用 gadget 注入
- `14725` 没起来，所以不会阻塞
- 这不是 `r0z` APatch 兼容失败，而是设备当前：
  - APatch 权限数据异常
  - `xgj` 目标 app 注入开关未正确写成 `true`

## 打包与发布

### 构建过程

- 第一次构建失败：
  - 原因：Gradle 实际跑在 Java 8
- 之后改用：

```bash
JAVA_HOME=/Users/fengyanrong/Library/Java/JavaVirtualMachines/jbr-17.0.12/Contents/Home ./gradlew zipRelease --no-daemon --console=plain
```

- 构建成功

### 本次发布版本

- `verName = v1.0.3`
- `versionCode = 4`

### 构建产物

- `module/build/outputs/release/r0z-v1.0.3-4-release.zip`

### 说明

- 当前没有 `module/private_key`
- 所以构建日志提示：
  - `this build will not be signed`

## 更新日志草案

### v1.0.3

#### Added

- Added APatch root implementation detection and status reporting in `r0zd`
- Added APatch-specific root policy parsing from `/data/adb/ap/package_config`
- Added APatch process flag propagation to the injector/runtime path
- Added APatch manager handling in app specialize flow
- Added APatch install-branch handling in the module installer

#### Changed

- Bumped module version to `v1.0.3` and `versionCode` to `4`
- Updated Web UI root description and module scanning logic to include APatch environments
- Added an APatch unmount compatibility path that currently follows the Magisk-style `/data/adb` cleanup strategy

#### Notes

- This release focuses on APatch compatibility for the r0z runtime path
- Existing KernelSU / Magisk behavior is preserved
- Release package naming follows `r0z-v1.0.3-4-<buildType>.zip`

## 时间线式对话摘录

### 1. 现象确认

- 用户反馈设备换成 APatch 后，`zygisk-gadget` 勾选 app 不阻塞
- 分析后确认 gadget 本身仍配置为 `on_load=wait`

### 2. 兼容性确认

- 用户要求检查当前 `r0z` 是否兼容 APatch
- 结论：原始仓库不兼容，只支持 `KernelSU / Magisk`

### 3. 方案设计

- 对照 APatch 官方实现，确定应新增独立 `APatch` root adapter

### 4. 直接修改

- 用户要求直接落代码并保存记录文档
- 完成 APatch 适配代码接入

### 5. 打包

- 用户要求打包
- 处理 Java 8 问题后，成功构建 release zip

### 6. ADB 验证

- 用户要求直接按条件检查设备
- 验证结果显示：
  - `r0z` 已识别 `APatch`
  - `r0z` 主链路正常
  - `zygisk-gadget` 当前目标 app 未启用注入

### 7. 文档补全

- 用户要求将修改内容和对话记录补入文档
- 已追加分析、ADB 实测、打包记录和时间线摘要
