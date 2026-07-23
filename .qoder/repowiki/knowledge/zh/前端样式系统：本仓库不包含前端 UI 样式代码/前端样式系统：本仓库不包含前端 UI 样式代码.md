---
kind: frontend_style
name: 前端样式系统：本仓库不包含前端 UI 样式代码
category: frontend_style
scope:
    - '**'
---

经全面检索，该仓库为 OpenIM Go SDK 核心库，采用纯 Go 语言实现，未包含任何前端样式相关代码与资源。仓库中不存在以下前端样式基础设施：
- 无 CSS/SCSS/Less/Sass 样式文件
- 无 Tailwind、Bootstrap、Ant Design 等前端样式框架或组件库引用
- 无主题（theme）定义、设计令牌（design tokens）或视觉规范文件
- 无 HTML 模板或前端页面结构
- wasm/cmd/static 目录仅包含浏览器运行时桥接脚本 wasm_exec.js，不含任何样式资源

README 文档中内联的 style 属性仅用于 GitHub 渲染时的 Markdown 排版，不属于项目样式体系。

因此，`frontend_style` 类别不适用于此仓库。