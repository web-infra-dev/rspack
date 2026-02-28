<picture>
  <img alt="Rspack Banner" src="https://assets.rspack.rs/rspack/rspack-banner.png">
</picture>

# Rspack

<p>
  <a href="https://discord.gg/79ZZ66GH9E"><img src="https://img.shields.io/badge/chat-discord-blue?style=flat-square&logo=discord&colorA=564341&colorB=EDED91" alt="discord channel" /></a>
  <a href="https://www.npmjs.com/package/@rspack/core?activeTab=readme"><img src="https://img.shields.io/npm/v/@rspack/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>
  <a href="https://crates.io/crates/rspack_core"><img src="https://img.shields.io/crates/v/rspack_core?style=flat-square&colorA=564341&colorB=EDED91" alt="crates version" /></a>
  <a href="https://npmcharts.com/compare/@rspack/core?minimal=true"><img src="https://img.shields.io/npm/dm/@rspack/core.svg?style=flat-square&colorA=564341&colorB=EDED91" alt="downloads" /></a>
  <a href="https://nodejs.org/en/about/previous-releases"><img src="https://img.shields.io/node/v/@rspack/core.svg?style=flat-square&colorA=564341&colorB=EDED91" alt="node version"></a>
  <a href="https://github.com/web-infra-dev/rspack/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square&colorA=564341&colorB=EDED91" alt="license" /></a>
  <a href="https://codspeed.io/web-infra-dev/rspack"><img src="https://img.shields.io/endpoint?url=https%3A%2F%2Fcodspeed.io%2Fbadge.json&style=flat-square&colorA=564341&colorB=EDED91" alt="codspeed" /></a>
</p>

[English](./README.md) | 简体中文

Rspack 是一个基于 Rust 编写的高性能 JavaScript 打包工具，它提供对 webpack 生态良好的兼容性，能够无缝替换 webpack，并提供闪电般的构建速度。

## ✨ 特性

- 🚀 **启动速度极快**: 基于 Rust 实现，构建速度极快，带给你极致的开发体验。
- ⚡ **闪电般的 HMR**: 内置增量编译机制，HMR 速度极快，完美胜任大型项目的开发。
- 📦 **兼容 webpack 生态**: 兼容 webpack 生态中的 plugin 和 loader，无缝衔接社区中沉淀的优秀库。
- 🎨 **模块联邦**: 为 Module Federation 提供一流的支持，助力开发规模化的 Web 应用。
- 🛠️ **默认生产优化**: 默认内置多种优化策略，如 Tree Shaking、代码压缩等等。
- 🎯 **框架无关**: 不和任何前端框架绑定，保证足够的灵活性。

请阅读 [Rspack 介绍](https://rspack.rs/zh/guide/start/introduction) 章节来了解更多。

## 🦀 Rstack

Rstack 是一个以 Rspack 为核心的 JavaScript 统一工具链，具有优秀的性能和一致的架构。

| 名称                                                  | 描述           | 版本                                                                                                                                                                             |
| ----------------------------------------------------- | -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [Rspack](https://github.com/web-infra-dev/rspack)     | 打包工具       | <a href="https://npmjs.com/package/@rspack/core"><img src="https://img.shields.io/npm/v/@rspack/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>     |
| [Rsbuild](https://github.com/web-infra-dev/rsbuild)   | 构建工具       | <a href="https://npmjs.com/package/@rsbuild/core"><img src="https://img.shields.io/npm/v/@rsbuild/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>   |
| [Rslib](https://github.com/web-infra-dev/rslib)       | 库开发工具     | <a href="https://npmjs.com/package/@rslib/core"><img src="https://img.shields.io/npm/v/@rslib/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>       |
| [Rspress](https://github.com/web-infra-dev/rspress)   | 静态站点生成器 | <a href="https://npmjs.com/package/@rspress/core"><img src="https://img.shields.io/npm/v/@rspress/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>   |
| [Rsdoctor](https://github.com/web-infra-dev/rsdoctor) | 构建分析工具   | <a href="https://npmjs.com/package/@rsdoctor/core"><img src="https://img.shields.io/npm/v/@rsdoctor/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a> |
| [Rstest](https://github.com/web-infra-dev/rstest)     | 测试框架       | <a href="https://npmjs.com/package/@rstest/core"><img src="https://img.shields.io/npm/v/@rstest/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>     |
| [Rslint](https://github.com/web-infra-dev/rslint)     | 代码检查工具   | <a href="https://npmjs.com/package/@rslint/core"><img src="https://img.shields.io/npm/v/@rslint/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>     |

## 快速上手

<p>
  <a target="_blank" href="https://stackblitz.com/fork/github/rstackjs/rspack-stackblitz-example">
    <img
      alt="Open in StackBlitz"
      src="https://developer.stackblitz.com/img/open_in_stackblitz.svg"
    />
  </a>
</p>

请阅读 [快速上手](https://rspack.rs/zh/guide/start/quick-start)。

## 参与贡献

请阅读 [贡献指南](./CONTRIBUTING.md) 来共同参与 Rspack 的建设。

### 行为准则

本仓库采纳了字节跳动的开源项目行为准则。请点击 [行为准则](./CODE_OF_CONDUCT.md) 查看更多的信息。

## 社区

- 可以在 [Discord](https://discord.gg/79ZZ66GH9E) 上和 Rspack Team 以及 Rspack 用户交流
- 也可以在 [飞书](https://applink.feishu.cn/client/chat/chatter/add_by_link?link_token=3c3vca77-bfc0-4ef5-b62b-9c5c9c92f1b4) 上和我们交流

## 链接

| 名称                                                                           | 描述                                                                         |
| ------------------------------------------------------------------------------ | ---------------------------------------------------------------------------- |
| [awesome-rstack](https://github.com/rstackjs/awesome-rstack)                   | 与 Rspack 相关的精彩内容列表                                                 |
| [agent-skills](https://github.com/rstackjs/agent-skills)                       | Rstack 的 Agent Skills 合集                                                  |
| [Rspack 2.x 文档](https://v2.rspack.rs/zh/)                                    | Rspack 2.x 版本的文档（Beta）                                                |
| [Rspack 1.x 文档](https://rspack.rs/zh/)                                       | Rspack 1.x 版本的文档（最新）                                                |
| [Rspack 0.x 文档](https://v0.rspack.rs/zh/)                                    | Rspack 0.x 版本的文档                                                        |
| [rspack-dev-server](https://github.com/web-infra-dev/rspack-dev-server)        | Rspack 的开发服务器                                                          |
| [rstack-examples](https://github.com/rstackjs/rstack-examples)                 | Rstack 的示例项目                                                            |
| [rspack-sources](https://github.com/rstackjs/rspack-sources)                   | Rust 版本的 [webpack-sources](https://www.npmjs.com/package/webpack-sources) |
| [rstack-design-resources](https://github.com/rstackjs/rstack-design-resources) | Rstack 的设计资源                                                            |

## 贡献者

<a href="https://github.com/web-infra-dev/rspack/graphs/contributors"><img src="https://opencollective.com/rspack/contributors.svg?width=890&button=false" /></a>

## 基准测试

参考 [基准测试](https://ecosystem-benchmark.rspack.rs/)。

## 致谢

感谢:

- [webpack 团队和社区](https://webpack.js.org/)创建了一个优秀的打包工具和丰富的生态。
- [@sokra](https://github.com/sokra) 在 [webpack](https://github.com/webpack/webpack) 项目上的出色工作。
- [@ScriptedAlchemy](https://github.com/ScriptedAlchemy) 创造了模块联邦，并帮助 Rspack 与社区建立联系。
- [SWC](https://swc.rs/) 项目（由 [@kdy1](https://github.com/kdy1) 创建），为 Rspack 的代码解析、转换和压缩提供了支持。
- [esbuild](https://github.com/evanw/esbuild) 项目（由 [@evanw](https://github.com/evanw) 创建），它启发了 Rspack 的并发架构。
- [NAPI-RS](https://github.com/napi-rs/napi-rs) 项目（由 [@Brooooooklyn](https://github.com/Brooooooklyn) 创建），为 Rspack 的 node-binding 实现提供了支持。
- [Parcel](https://github.com/parcel-bundler/parcel) 项目（由 [@devongovett](https://github.com/devongovett)创建），它是 Rust Bundler 的先行探索者并启发了 Rspack 的增量构建架构。
- [Vite](https://github.com/vitejs/vite) 由[尤雨溪](https://github.com/yyx990803)创建，它和 Rollup 社区的兼容性设计启发了 Rspack 和 webpack 社区的兼容设计。
- `rolldown-legacy` 项目，它是 [rolldown](https://github.com/rolldown) 项目的前身，它探索了使用 Rust 构建高性能 Bundler + 兼容 Rollup API 的可能性，启发了 Rspack 的设计方向。
- [html-webpack-plugin](https://github.com/jantimon/html-webpack-plugin) 项目（由 [@jantimon](https://github.com/jantimon) 创建），Rspack 的 `@rspack/html-plugin` 是 [html-webpack-plugin](https://github.com/jantimon/html-webpack-plugin) 的一个 fork 来避免使用在 Rspack 中尚未支持的 webpack API。
- [Turbopack](https://github.com/vercel/turbo) 项目，它启发了 Rspack 里基于 AST 的路径重写逻辑。
- [react-refresh-webpack-plugin](https://github.com/pmmmwh/react-refresh-webpack-plugin) 项目（由 [@pmmmwh](https://github.com/pmmmwh) 创建），它启发了 Rspack 内的 ReactRefreshPlugin 实现。
- [prefresh](https://github.com/preactjs/prefresh) 项目（由 [@Jovi De Croock](https://github.com/JoviDeCroock) 创建），它启发了 Rspack 内的 PreactRefreshPlugin 实现。
- [mini-css-extract-plugin](https://github.com/webpack/mini-css-extract-plugin) 项目（由 [@sokra](https://github.com/sokra) 创建），它启发了 Rspack 内的 CssExtractPlugin 实现。
- [copy-webpack-plugin](https://github.com/webpack/copy-webpack-plugin) 项目（由 [@kevlened](https://github.com/kevlened) 创建），它启发了 Rspack 内的 CopyPlugin 实现。
- [webpack-subresource-integrity](https://github.com/waysact/webpack-subresource-integrity) 项目（由 [@jscheid](https://github.com/jscheid) 创建），它启发了 Rspack 内的 SubresourceIntegrityPlugin 实现。
- [circular-dependency-plugin](https://github.com/aackerman/circular-dependency-plugin) 项目（由 [@aackerman](https://github.com/aackerman) 创建），它启发 Rspack 中循环依赖插件的实现。
- [tracing-chrome](https://github.com/thoren-d/tracing-chrome) 项目（由 [thoren-d](https://github.com/thoren-d) 创建），它启发 Rspack tracing 功能的实现。

## License

Rspack 项目基于 [MIT 协议](https://github.com/web-infra-dev/rspack/blob/main/LICENSE)，请自由地享受和参与开源。
