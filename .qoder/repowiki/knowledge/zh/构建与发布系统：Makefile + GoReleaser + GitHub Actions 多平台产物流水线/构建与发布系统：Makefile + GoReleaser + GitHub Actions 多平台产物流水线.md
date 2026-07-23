---
kind: build_system
name: 构建与发布系统：Makefile + GoReleaser + GitHub Actions 多平台产物流水线
category: build_system
scope:
    - '**'
source_files:
    - Makefile
    - .goreleaser.yaml
    - .github/workflows/go-build-test.yml
    - .github/workflows/sdk-releaser.yml
    - wasm/cmd/Makefile
    - .golangci.yml
---

## 1. 使用的系统与工具链
- **构建编排**：根目录 `Makefile` 提供统一的 build/lint/test/release 入口，封装 go build、gomobile bind、WASM 编译等命令。
- **跨平台打包**：`.goreleaser.yaml` 负责标准二进制归档、Changelog 生成与 GitHub Release 上传；Android/iOS/WASM 产物由专用 workflow 产出并单独上传。
- **CI/CD**：GitHub Actions 提供两套流水线——`go-build-test.yml`（PR/push 触发）和 `sdk-releaser.yml`（release 或手动触发），分别承担构建测试与多平台发布。
- **版本注入**：通过 `-ldflags -X` 将 Git 描述信息注入 `version` 包，供运行时输出；Android/iOS/WASM 在 release 前会写入 `version/version` 文件。
- **依赖管理**：Go modules（`go.mod`/`go.sum`），CI 中统一执行 `go mod tidy`。

## 2. 关键文件与位置
- 根构建脚本：`Makefile`
- 通用发布配置：`.goreleaser.yaml`
- CI 工作流：
  - `.github/workflows/go-build-test.yml`（构建+集成测试）
  - `.github/workflows/sdk-releaser.yml`（Android/iOS/WASM 发布）
- WASM 子构建：`wasm/cmd/Makefile`、`wasm/cmd/main.go`
- 版本常量注入目标包：`version/version.go`、`pkg/version/`（被 ldflags 注入）
- 代码质量：`.golangci.yml`、`.codecov.yml`

## 3. 架构与约定
### 3.1 构建目标分层
- **本地开发**：`make build` 按当前 GOOS/GOARCH 产出 `_output/bin/openim-sdk-core-<os>-<arch>`；`make wasm` 编译 `wasm/cmd/main.go` 为 `openIM.wasm`。
- **交叉编译**：`make build-multiple` 遍历 linux/amd64、linux/arm64；iOS/Android 通过 `make ios` / `make android` 调用 gomobile bind 生成 xcframework/aar。
- **容器镜像**：`make docker-build` / `docker-push` / `docker-buildx-push` 使用仓库根 Dockerfile（未在本仓库展示，但 Makefile 已暴露目标）。
- **发布流程**：`make release` → `scripts/release.sh` → GoReleaser 打包归档；Android/iOS/WASM 由 `sdk-releaser.yml` 并行构建并上传到对应 tag 的 Release。

### 3.2 版本与元数据注入策略
- 运行期版本：`VERSION := $(shell git describe --tags --always --match="v*" --dirty | sed 's/-/./g')`，配合 `-ldflags -X version.GitVersion=...` 注入。
- 发布版本号：release workflow 在构建前写 `version/version` 文件，供 Android/iOS/WASM 构建读取；GoReleaser 基于 Git tag 自动推断 prerelease。

### 3.3 多平台产物矩阵
| 平台 | 构建方式 | 产物 | 上传位置 |
|---|---|---|---|
| Linux/macOS/Windows 二进制 | `go build` + Goreleaser archives | tar.gz/zip 归档 | GitHub Releases (Goreleaser) |
| Android AAR | `make android` (gomobile bind) | `open_im_sdk.aar` | SDK Releaser job 打包 zip 后上传 |
| iOS xcframework | `make ios` (gomobile bind) | `build/OpenIMCore.xcframework` | SDK Releaser job 打包 zip 后上传 |
| WebAssembly | `wasm/cmd/Makefile` | `openIM.wasm` + `static/wasm_exec.js` | SDK Releaser job 打包 zip 后上传 |

### 3.4 代码质量门禁
- `make style` 串联 `fmt` / `vet` / `lint`，lint 使用 `.golangci.yml` 规则集。
- `make test` / `cover` 执行单元测试；`go-build-test.yml` 还拉取 open-im-server 启动服务，运行 `integration_test` 做端到端验证。

## 4. 开发者应遵循的规则
1. **新增可执行目标**：优先在根 `Makefile` 中以 `##` 注释声明，便于 `make help` 发现；复杂平台特定逻辑放入子 `Makefile`（如 `wasm/cmd/Makefile`）。
2. **跨平台构建**：使用 `GOOS`/`GOARCH` 环境变量驱动，不要硬编码平台；需要 C/C++ 交叉工具链时参考现有 arm64 示例设置 `CC`/`CXX`。
3. **版本注入**：如需在二进制中暴露版本信息，通过 `-ldflags -X` 注入 `version` 包字段；发布时确保 `version/version` 文件存在或被 workflow 创建。
4. **新平台产物**：若新增平台（如 Windows ARM64），同步更新 `OSES`/`ARCHS` 矩阵以及 `sdk-releaser.yml` 对应 job，并在 `archives.name_template` 中处理命名。
5. **依赖与工具**：所有第三方工具通过 `tools.verify.%` / `install.*` 目标安装到 `_output/tools`，避免污染全局 GOPATH；新增工具需在此处注册。
6. **CI 一致性**：本地 `make` 行为应与 CI 步骤保持一致（例如 `go mod tidy && go generate ./...` 应在提交前执行），避免 PR 构建失败。
7. **Release 规范**：打 `v*` tag 触发 GoReleaser；Android/iOS/WASM 可通过 `workflow_dispatch` 指定 `tag_name` 手动触发。