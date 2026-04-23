# r0z 第二版（v1.0.1 / verCode=2）相对第一版（v1.0.0）差异记录

## 1. 比较基线与范围
- 基线版本：`v1.0.0`（tag）
- 当前版本目标：`v1.0.1`，`verCode=2`
- 比较方式：`git diff v1.0.0` + 当前工作区未提交新增/删除文件
- 统计（相对 `v1.0.0`）：
  - 已跟踪文件差异：32 个文件
  - 代码行变化：`359 insertions(+), 590 deletions(-)`
  - 额外未跟踪新增：`loader/src/native_bridge/loader.cpp`、`loader/src/native_bridge/payload.cpp`、`zygiskd/src/r0zd.rs`

## 2. 第二版核心变化（相对第一版）

### 2.1 命名与版本体系统一
- 模块与工程命名从 `r0zygisk` 收敛为 `r0z`：
  - `moduleId`：`r0zygisk -> r0z`
  - `moduleName`：`r0zygisk -> r0z`
  - Gradle rootProject 名称：`r0zygisk -> r0z`
  - Android namespace：`com.r0ysue.r0zygisk -> com.r0ysue.r0z`
- 版本更新：
  - `verName: v1.0.0 -> v1.0.1`
  - `verCode: 1 -> 2`

### 2.2 核心运行链路从 ptrace 方案向 Native Bridge 方案切换
- 新增 native bridge loader/payload 组件：
  - `libzn_loader.so`（native bridge 入口）
  - `libpayload.so`
- `post-fs-data.sh` / `service.sh` 中新增（或强化）以下行为：
  - 启动 `r0zd daemon`
  - 设置 `ro.dalvik.vm.native.bridge=libzn_loader.so`
  - 等待 `cp32.sock/cp64.sock` 就绪，降低 zygote 初始化竞争风险
- WebUI 与状态字段适配 native bridge 模式（不再以“按钮控制 ptracer”为主）

### 2.3 注入库与二进制命名调整
- 注入库命名：`libzygisk.so -> libr0zgk.so`
- daemon 二进制命名：`zygiskd -> r0zd`
- 控制脚本命名：`zygisk-ctl.sh -> r0z-ctl.sh`

### 2.4 打包与安装路径重构
- 安装路径由 `lib/lib64` 改为更标准的：
  - `$MODPATH/system/lib`
  - `$MODPATH/system/lib64`
- `customize.sh` 适配多 ABI 与 32/64 混合设备逻辑
- `module/build.gradle.kts` 的签名映射、拷贝清单、产物映射全部同步到新命名和新路径

### 2.5 Rust 子项目重命名与状态输出变更
- 子项目逻辑名：`:zygiskd -> :r0zd`（目录仍为 `zygiskd`）
- Cargo 包名：`zygiskd -> r0zd`
- `main.rs` 增加多入口参数模式（`daemon/service-stage/companion/version/root`）
- daemon 状态输出更新为 native bridge 导向（写入 `status.json`）

### 2.6 文档与 UI 文案更新
- README / README.zh-CN 多处术语由 “Zygisk” 调整为 “R0z” 语义
- WebUI 标题、说明、状态文案同步到 `r0z` + native bridge 运行模型
- 新增第二版变更记录到 `CHANGELOG.md`

## 3. 与第一版差异矩阵

| 维度 | 第一版（v1.0.0） | 第二版（v1.0.1） |
|---|---|---|
| 模块标识 | `r0zygisk` | `r0z` |
| daemon 名称 | `zygiskd` | `r0zd` |
| 注入库名 | `libzygisk.so` | `libr0zgk.so` |
| 控制脚本 | `zygisk-ctl.sh` | `r0z-ctl.sh` |
| 主要注入触发 | ptracer + 传统链路 | native bridge (`libzn_loader.so`) + daemon 常驻 |
| 产物目录重点 | `lib/lib64` | `system/lib` / `system/lib64` |
| WebUI 交互重点 | 追踪器控制按钮 | 状态观测与 native bridge 状态展示 |
| 版本号 | `v1.0.0 / code 1` | `v1.0.1 / code 2` |

## 4. 文件级修改清单（相对 v1.0.0）

### 4.1 构建与工程配置
- `build.gradle.kts`
- `settings.gradle.kts`
- `loader/build.gradle.kts`
- `module/build.gradle.kts`
- `zygiskd/build.gradle.kts`
- `zygiskd/Cargo.toml`

### 4.2 Loader / 注入层
- `loader/src/CMakeLists.txt`
- `loader/src/common/daemon.cpp`
- `loader/src/include/daemon.h`
- `loader/src/include/native_bridge_callbacks.h`
- `loader/src/include/logging.h`
- `loader/src/injector/entry.cpp`
- `loader/src/injector/hook.cpp`
- `loader/src/ptracer/main.cpp`
- `loader/src/ptracer/monitor.cpp`
- `loader/src/ptracer/ptracer.cpp`
- `loader/src/ptracer/utils.hpp`

### 4.3 模块打包/安装脚本与 WebUI
- `module/src/customize.sh`
- `module/src/post-fs-data.sh`
- `module/src/service.sh`
- `module/src/module.prop`
- `module/src/webroot/app.js`
- `module/src/webroot/index.html`
- `module/src/r0z-ctl.sh`（新增）
- `module/src/zygisk-ctl.sh`（删除）

### 4.4 Rust daemon 层
- `zygiskd/src/main.rs`
- `zygiskd/src/constants.rs`
- `zygiskd/src/companion.rs`
- `zygiskd/src/zygiskd.rs`（删除）
- `zygiskd/src/r0zd.rs`（当前工作区新增，用于承接重命名后的实现）

### 4.5 文档
- `CHANGELOG.md`
- `README.md`
- `README.zh-CN.md`

### 4.6 当前工作区未跟踪新增（未纳入 tag 对比统计）
- `loader/src/native_bridge/loader.cpp`
- `loader/src/native_bridge/payload.cpp`
- `zygiskd/src/r0zd.rs`

## 5. 第二版发布说明（可直接用于 Release）

### 5.1 主要改进
- 完成从 `r0zygisk` 到 `r0z` 的命名与构建链路统一
- 引入 native bridge loader 方案，降低传统 ptrace 依赖
- 注入库更名为 `libr0zgk.so`，并完成加载、安装、打包、校验链路全量同步
- daemon 更名 `r0zd`，控制脚本切换为 `r0z-ctl`
- WebUI 聚焦“状态观测”和 native bridge 状态诊断

### 5.2 兼容性说明
- Root 环境最低要求保持既有约束（KernelSU / Magisk 版本门槛未放宽）
- 若缺少 `module/private_key` 与 `module/public_key`，构建产物仍为未签名（与第一版一致）

### 5.3 升级建议
- 从第一版升级时，建议完整重启一次设备，确保 `ro.dalvik.vm.native.bridge` 与 daemon 状态稳定生效
- 使用新版包名规则：`r0z-v1.0.1-2-<buildType>.zip`

## 6. 备注
- 本文档记录的是“当前第二版工作区”相对于 `v1.0.0` 的完整差异。
- 其中包含了尚未提交到 Git 的新增文件（见 4.6），提交发布前请确认这些文件已纳入版本控制。
