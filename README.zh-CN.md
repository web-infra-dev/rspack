<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://lf3-static.bytednsdoc.com/obj/eden-cn/rjhwzy/ljhwZthlaukjlkulzlp/rspack-banner-1610-dark.png">
  <img alt="Rspack Banner" src="https://lf3-static.bytednsdoc.com/obj/eden-cn/rjhwzy/ljhwZthlaukjlkulzlp/rspack-banner-1610.png">
</picture>

<h2 align="center">基于 Rust 的高性能模块打包工具</h2>

<p align="center">
  <a href="https://www.npmjs.com/package/@rspack/core?activeTab=versions">
   <img src="https://img.shields.io/npm/v/@rspack/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" />
  </a>
  <a href="https://npmcharts.com/compare/@rspack/core?minimal=true">
    <img src="https://img.shields.io/npm/dm/@rspack/core.svg?style=flat-square&colorA=564341&colorB=EDED91" alt="downloads" />
  </a>
  <a href="https://github.com/web-infra-dev/rspack/blob/main/LICENSE">
    <img src="https://img.shields.io/npm/l/@rspack/core?style=flat-square&colorA=564341&colorB=EDED91" alt="license" />
  </a>
</p>

[English](./README.md) | 简体中文

## ✨ 特性

- 🚀 **启动速度极快**: 基于 Rust 实现，构建速度极快，带给你极致的开发体验。
- ⚡ **闪电般的 HMR**: 内置增量编译机制，HMR 速度极快，完全胜任大型项目的开发。
- 📦 **兼容 webpack 生态**: 针对 webpack 的架构和生态进行兼容，无需从头搭建生态。
- 🎨 **内置常见构建能力**: 对 TypeScript、JSX、CSS、CSS Modules、Sass 等提供开箱即用的支持。
- 🛠️ **默认生产优化**: 默认内置多种优化策略，如 Tree Shaking、代码压缩等等。
- 🎯 **框架无关**: 不和任何前端框架绑定，保证足够的灵活性。

请阅读 [Rspack 介绍](https://rspack.dev/zh/guide/introduction.html) 章节来了解更多。

## 快速上手

- [快速上手](https://rspack.dev/zh/guide/quick-start.html)

## 参与贡献

> **Rspack 项目目前仍在积极开发中，仍然缺失了很多的 webpack 的插件 Hook 以及一些 API 目前可能不够稳定,我们正在努力补齐这些 API，并不断地提升 Rspack 的稳定性，如果您有任何建议或意见，欢迎随时提交 PR 或 Issue，我们将不胜感激。**

请阅读 [贡献指南](./CONTRIBUTING.md) 来共同参与 Rspack 的建设。

### 行为准则

本仓库采纳了字节跳动的开源项目行为准册。 请点击[行为准则](./CODE_OF_CONDUCT.md)查看更多的信息。

## 链接

| 名称                                                                                    | 描述                                                                         |
| --------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| [Rspack 文档](https://rspack.dev)                                                       | Rspack 官方文档                                                              |
| [Rspack 文档仓库](https://github.com/web-infra-dev/rspack-website)                      | Rspack 官方文档的代码仓库                                                    |
| [rspack-sources](https://github.com/web-infra-dev/rspack-sources)                       | Rust 版本的 [webpack-sources](https://www.npmjs.com/package/webpack-sources) |
| [rspack-migration-showcase](https://github.com/web-infra-dev/rspack-migration-showcase) | 迁移到 Rspack 的示例项目                                                     |

## 致谢

感谢:

- [webpack 团队和社区](https://webpack.js.org/) 创建了一个优秀的打包工具和丰富的生态。
- [@sokra](https://github.com/sokra) 在 [webpack](https://github.com/webpack/webpack) 项目上的出色工作。
- [@ScriptedAlchemy](https://github.com/ScriptedAlchemy) 创造了模块联邦，并帮助 Rspack 与社区建立联系。
- [SWC](https://swc.rs/) 项目（由 [@kdy1](https://github.com/kdy1) 创建）, 为 Rspack 的代码解析、转换和压缩提供了支持。
- [esbuild](https://github.com/evanw/esbuild) 项目（由 [@evanw](https://github.com/evanw) 创建）, 它启发了 Rspack 的并发架构。
- [NAPI-RS](https://github.com/napi-rs/napi-rs) 项目（由 [@Brooooooklyn](https://github.com/Brooooooklyn) 创建），为 Rspack 的 node-binding 实现提供了支持。
- [Parcel](https://github.com/parcel-bundler/parcel) 项目(由 [@devongovett](https://github.com/devongovett)创建), 它是 Rust Bundler 的先行探索者并启发了 Rspack 的增量构建架构.
- [Vite](https://github.com/vitejs/vite) 由[尤雨溪](https://github.com/yyx990803)创建，它和 rollup 社区的兼容性设计启发了 Rspack 和 Webpack 社区的兼容设计。
- [Rolldown](https://github.com/rolldown-rs/rolldown) 项目(由 [Rolldown 团队](https://github.com/sponsors/rolldown-rs)创建)，它探索了使用 Rust 构建高性能 Bundler + 兼容 Rollup API 的可能性，启发了 Rspack 的设计方向。
- [html-webpack-plugin](https://github.com/jantimon/html-webpack-plugin) 项目（由 [@jantimon](https://github.com/jantimon) 创建）, Rspack 的 `@rspack/html-plugin` 是 [[html-webpack-plugin](https://github.com/jantimon/html-webpack-plugin) 的一个 fork 来避免使用在 Rspack 中尚未支持的 webpack API。
- [Turbopack](https://github.com/vercel/turbo) 项目，它启发了 Rspack 里基于 AST 的路径重写逻辑。

## License

Rspack 项目基于 [MIT 协议](https://github.com/web-infra-dev/rspack/blob/main/LICENSE)，请自由地享受和参与开源。
