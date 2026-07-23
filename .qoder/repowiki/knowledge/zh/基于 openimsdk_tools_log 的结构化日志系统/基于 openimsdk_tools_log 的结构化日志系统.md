---
kind: logging_system
name: 基于 openimsdk/tools/log 的结构化日志系统
category: logging_system
scope:
    - '**'
source_files:
    - internal/third/log.go
    - pkg/db/db_init.go
    - open_im_sdk/caller.go
    - open_im_sdk/apicb.go
    - open_im_sdk/em.go
    - internal/conversation_msg/api.go
    - pkg/common/trigger_channel.go
    - pkg/converter/conversation.go
    - pkg/db/chat_log_model.go
    - wasm/event_listener/caller.go
    - wasm/exec/executor.go
---

## 1. 使用的系统与框架
- 核心日志库：github.com/openimsdk/tools/log，提供结构化日志 API（ZDebug、ZInfo、ZWarn、ZError）。
- SDK 内部通过 internal/third/log.go 的 printLog 调用 log.SDKLog(ctx, level, file, line, msg, err, keysAndValues) 将日志输出到本地文件，并支持打包上传至服务端。
- GORM SQL 层使用 gorm.io/gorm/logger 控制 SQL 日志级别（Info / Silent），由配置项 ServerConf.LogLevel 驱动。

## 2. 关键文件与包
- internal/third/log.go：日志文件扫描、按行截取、zip 压缩、上传到第三方存储，以及 printLog 桥接 log.SDKLog。
- pkg/db/db_init.go：根据 ServerConf.LogLevel 设置 GORM logger 级别（Info/Silent）。
- open_im_sdk/caller.go、open_im_sdk/apicb.go、open_im_sdk/em.go：门面层广泛使用 log.Z* 记录函数调用、回调错误等。
- internal/conversation_msg/api.go、conversation.go、notification.go 等：业务域大量使用 log.ZDebug/ZWarn/ZError 记录消息收发、会话同步、通知处理过程。
- pkg/common/trigger_channel.go、pkg/converter/*.go、pkg/db/chat_log_model.go：基础设施层统一通过 openimsdk/tools/log 输出。
- wasm/event_listener/caller.go、wasm/exec/executor.go：WASM 平台同样复用同一日志包。
- cmd/sdk/main.go：示例程序用 fmt.Println 打印运行信息（非正式日志）。

## 3. 架构与约定
- 统一入口：所有模块直接依赖 github.com/openimsdk/tools/log，不自行实现 logger；SDK 内部通过 third.printLog 间接写入本地日志文件。
- 结构化字段：日志以 key-value 形式附加上下文（如 operationID、userID、conversationID、clientMsgID、serverMsgID、time cost 等），便于检索与聚合。
- 日志级别策略：
  - ZDebug：调试/跟踪路径，默认可能关闭。
  - ZInfo：正常业务流程关键点（函数入参/出参、成功事件）。
  - ZWarn：可恢复异常或降级场景（上传失败、解析错误、重试）。
  - ZError：不可恢复错误或需要告警的错误。
- SQL 日志开关：通过 ServerConf.LogLevel 映射到 logger.Info 或 logger.Silent，避免生产环境输出过多 SQL。
- 日志文件命名与轮转：文件名形如 open-im-sdk-core.yyyy-mm-dd，checkLogPath 校验日期后缀；上传成功后删除旧文件并截断当前文件，防止无限增长。
- 远程上报链路：uploadLogs → zip 压缩 → 分片上传到对象存储 → 调用 api.UploadLogs 上报 URL 及元数据（平台、框架、版本、原因等）。

## 4. 开发者应遵循的规则
- 禁止直接使用 fmt.Print* 作为运行时日志，统一使用 log.ZDebug/ZInfo/ZWarn/ZError。
- 始终携带 context：第一个参数为 ctx，以便自动注入 operationID 等追踪字段。
- 键值对风格：错误必须传入 error 参数，其他上下文以 key, value 成对追加，保持结构化。
- 合理选择级别：仅对真正需要关注的点使用 ZWarn/ZError，避免污染日志；性能相关耗时建议用 ZInfo + time.Since 记录。
- 不要硬编码日志路径：日志目录来自初始化配置，上传逻辑依赖 open-im-sdk-core.*.yyyy-mm-dd 命名规范，不得随意更改。
- GORM SQL 日志：仅在开发/排障时开启 Info 级别，生产应保持 Silent。
- WASM 平台：同样使用 openimsdk/tools/log，无需额外适配。