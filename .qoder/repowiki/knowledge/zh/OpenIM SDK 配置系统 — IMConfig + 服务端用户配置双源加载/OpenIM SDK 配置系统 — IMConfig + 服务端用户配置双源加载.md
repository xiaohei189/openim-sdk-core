---
kind: configuration_system
name: OpenIM SDK 配置系统 — IMConfig + 服务端用户配置双源加载
category: configuration_system
scope:
    - '**'
source_files:
    - sdk_struct/sdk_struct.go
    - open_im_sdk/init_login.go
    - open_im_sdk/userRelated.go
    - pkg/cliconf/client_config.go
    - pkg/cliconf/global.go
    - cmd/sdk/main.go
    - integration_test/internal/config/config.go
---

## 1. 采用的配置体系与工具

- **进程级初始化配置**：通过 `sdk_struct.IMConfig`（JSON 结构体）由调用方构造，以 JSON 字符串形式传入 `open_im_sdk.InitSDK(...)`，内部反序列化为 `IMConfig` 后用于日志、网络地址、数据目录等全局初始化。
- **运行时用户级配置**：通过 `pkg/cliconf` 包从 OpenIM 服务端拉取“用户客户端配置”（key-value），本地缓存并支持异步刷新，当前仅解析 `CONVERSATION_ACTIVE_NUM` 字段。
- **命令行参数与环境变量**：示例程序 `cmd/sdk/main.go` 使用标准库 `flag` 提供 `-api / -ws / -phone / -password / -area / -account-api / -data-dir` 等入口参数，并通过 `SDK_PASSWORD`、`SDK_DEBUG` 环境变量覆盖。
- **测试/集成用例配置**：`integration_test/internal/config/config.go` 中定义 `GetConf()` 返回 `sdk_struct.IMConfig`，供集成测试统一注入。

## 2. 关键文件与包

- `sdk_struct/sdk_struct.go`：定义 `IMConfig` 结构体及消息/元素等公共数据结构，是 SDK 对外暴露的“配置契约”。
- `open_im_sdk/init_login.go`：`InitSDK(listener, operationID, config string)` 入口，负责反序列化 `IMConfig`、校验协议前缀、初始化日志、再委托给 `UserContext.InitSDK`。
- `open_im_sdk/userRelated.go`：`UserContext.InitSDK(config *IMConfig, listener)` 实际落盘持久化配置、保存 `u.info.IMConfig`，并提供 `ImConfig()` 读取器。
- `pkg/cliconf/client_config.go` + `global.go`：实现基于 `atomic.Pointer` 的用户级配置缓存，通过 `api.ExtractField(ctx, api.UserClientConfig.Invoke, ...)` 拉取服务端 key-value 配置，提供 `GetClientConfig(ctx)` 和 `ClearConfig()`。
- `cmd/sdk/main.go`：演示如何组装 `IMConfig`、用 `flag` 解析参数、调用 `InitSDK` 并完成登录流程。
- `integration_test/internal/config/config.go`：测试侧构建 `IMConfig` 的辅助函数。

## 3. 架构与设计约定

### 3.1 两层配置模型

| 层次 | 来源 | 生命周期 | 典型用途 |
| --- | --- | --- | --- |
| 进程级 `IMConfig` | 调用方构造 → JSON → `InitSDK` | 进程启动时一次初始化，可被 `UnInitSDK` 清理 | API/WSS 地址、数据目录、日志级别、平台标识等 |
| 用户级 ClientConfig | 登录后从服务端 `UserClientConfig` 接口拉取 | 按用户维度缓存，登录切换时 `SetLoginUserID` 触发清空并异步重拉 | 会话活跃数、灰度开关等运行时可调参数 |

- 进程级配置在 `InitSDK` 阶段即完成校验与日志初始化；用户级配置在 `Login` 时设置 `cliconf.SetLoginUserID`，随后业务域按需 `cliconf.GetClientConfig(ctx)` 获取。
- `cliconf` 采用“首次懒加载 + 超时重试 + CAS 替换”模式：`getCurrConfig` 最多重试 10 次，每次失败会关闭等待 channel 并记录错误，避免 goroutine 泄漏。

### 3.2 配置项规范

- `IMConfig` 所有字段均带 json tag，要求调用方保证必填字段（如 `PlatformID != 0`、`ApiAddr` 含 `http`、`WsAddr` 含 `ws`）。
- 新增配置项应优先放在 `IMConfig` 中作为进程级常量，或放入服务端 `UserClientConfig` 作为用户级动态开关；两者命名需保持语义清晰，避免硬编码魔法值。

### 3.3 与日志系统的耦合

- `InitSDK` 直接调用 `log.InitLoggerFromConfig`，将 `IMConfig.LogLevel`、`LogFilePath`、`IsLogStandardOutput`、`LogRemainCount` 等映射到底层 logger。
- 这意味着修改日志行为必须通过 `IMConfig`，不应在业务层单独初始化 logger。

## 4. 开发者应遵循的规则

1. **不要绕过 `InitSDK`**：所有进程级配置必须经由 `open_im_sdk.InitSDK` 传入 JSON 字符串，禁止在业务代码中直接读写全局配置。
2. **新增进程级配置项**：在 `sdk_struct.IMConfig` 增加字段并在 `InitSDK` 中做必要校验（协议前缀、非空等），同时在 `UserContext.InitSDK` 中持久化。
3. **新增用户级动态配置**：在服务端添加对应 key，并在 `pkg/cliconf/client_config.go` 的 `parseServerUserConfig` 中解析默认值；如需变更默认值，调整 `*Default` 常量。
4. **跨模块访问用户配置**：统一通过 `cliconf.GetClientConfig(ctx)` 获取，不要在业务域内自行发起 HTTP 请求拉取。
5. **登录切换时清理缓存**：调用 `cliconf.SetLoginUserID(newUID)` 会自动 `ClearConfig`，确保新用户的配置异步重拉。
6. **示例与测试对齐**：参考 `cmd/sdk/main.go` 组装 `IMConfig`，集成测试通过 `integration_test/internal/config/config.go` 的 `GetConf()` 复用同一份配置结构。
7. **环境变量仅用于调试/覆盖**：`SDK_PASSWORD`、`SDK_DEBUG` 等仅在示例程序中生效，不应成为生产环境配置的主要通道。
