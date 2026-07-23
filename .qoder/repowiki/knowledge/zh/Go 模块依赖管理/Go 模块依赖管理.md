---
kind: dependency_management
name: Go 模块依赖管理
category: dependency_management
scope:
    - '**'
source_files:
    - go.mod
    - go.sum
---

本仓库采用 Go Modules 进行依赖管理，核心约定如下：

- **模块路径与版本**：`module github.com/openimsdk/openim-sdk-core/v3`，遵循语义化主版本后缀 v3。
- **Go 版本锁定**：`go 1.24.7`，通过 `go.mod` 的 `go` 指令固定工具链版本。
- **依赖声明方式**：所有第三方库在 `require` 块中显式声明，包括直接依赖（如 `github.com/coder/websocket`、`gorm.io/gorm`）和间接依赖（带 `// indirect` 注释），由 `go mod tidy` 维护。
- **私有协议包**：依赖同组织的 `github.com/openimsdk/protocol` 与 `github.com/openimsdk/tools`，均为 alpha 预发布版本，表明 SDK 与 OpenIM 服务端协议强绑定且随其迭代。
- **锁文件**：使用标准 `go.sum` 记录每个依赖的校验和，未启用 vendor 目录，依赖从远程 GOPROXY 拉取。
- **构建产物无关性**：无 `.golangci.yml` 之外的依赖相关配置，也未见 GOPRIVATE、replace 或自定义私有源设置，默认走公共代理。

开发者应遵循的规则：
- 新增依赖后执行 `go mod tidy` 同步 `go.mod` 与 `go.sum`。
- 升级 `openimsdk/protocol`、`openimsdk/tools` 时需同步验证 API 兼容性。
- 避免引入非必要的间接依赖，保持最小依赖集。