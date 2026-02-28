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

English | [简体中文](./README.zh-CN.md)

Rspack is a high performance JavaScript bundler written in Rust. It offers strong compatibility with the webpack ecosystem, allowing for seamless replacement of webpack, and provides lightning fast build speeds.

## ✨ Features

- 🚀 **Fast Startup**: Based on Rust, the build speed is extremely fast, bringing you the ultimate development experience.
- ⚡ **Lightning HMR**: With a built-in incremental compilation mechanism, HMR is extremely fast and fully capable of developing large-scale projects.
- 📦 **Webpack Compatible**: Compatible with plugins and loaders in the webpack ecosystem, seamlessly integrating excellent libraries built by the community.
- 🎨 **Module Federation**: Provide first-class support for Module Federation to facilitate the development of large-scale web applications.
- 🛠️ **Production Optimization**: Various optimization strategies are built in by default, such as tree shaking, minification, etc.
- 🎯 **Framework Agnostic**: Not bound to any frontend framework, ensuring enough flexibility.

Read [Introduction](https://rspack.rs/guide/start/introduction) for details.

## 🦀 Rstack

Rstack is a unified JavaScript toolchain centered on Rspack, with high performance and consistent architecture.

| Name                                                  | Description              | Version                                                                                                                                                                          |
| ----------------------------------------------------- | ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [Rspack](https://github.com/web-infra-dev/rspack)     | Bundler                  | <a href="https://npmjs.com/package/@rspack/core"><img src="https://img.shields.io/npm/v/@rspack/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>     |
| [Rsbuild](https://github.com/web-infra-dev/rsbuild)   | Build tool               | <a href="https://npmjs.com/package/@rsbuild/core"><img src="https://img.shields.io/npm/v/@rsbuild/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>   |
| [Rslib](https://github.com/web-infra-dev/rslib)       | Library development tool | <a href="https://npmjs.com/package/@rslib/core"><img src="https://img.shields.io/npm/v/@rslib/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>       |
| [Rspress](https://github.com/web-infra-dev/rspress)   | Static site generator    | <a href="https://npmjs.com/package/@rspress/core"><img src="https://img.shields.io/npm/v/@rspress/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>   |
| [Rsdoctor](https://github.com/web-infra-dev/rsdoctor) | Build analyzer           | <a href="https://npmjs.com/package/@rsdoctor/core"><img src="https://img.shields.io/npm/v/@rsdoctor/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a> |
| [Rstest](https://github.com/web-infra-dev/rstest)     | Testing framework        | <a href="https://npmjs.com/package/@rstest/core"><img src="https://img.shields.io/npm/v/@rstest/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>     |
| [Rslint](https://github.com/web-infra-dev/rslint)     | Linter                   | <a href="https://npmjs.com/package/@rslint/core"><img src="https://img.shields.io/npm/v/@rslint/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>     |

## Getting started

<p>
  <a target="_blank" href="https://stackblitz.com/fork/github/rstackjs/rspack-stackblitz-example">
    <img
      alt="Open in StackBlitz"
      src="https://developer.stackblitz.com/img/open_in_stackblitz.svg"
    />
  </a>
</p>

See [Quick start](https://rspack.rs/guide/start/quick-start).

## Contribution

Please read the [contributing guide](./CONTRIBUTING.md) and let's build Rspack together.

### Code of conduct

This repo has adopted the ByteDance Open Source Code of Conduct. Please check [Code of conduct](./CODE_OF_CONDUCT.md) for more details.

## Community

Come chat with us on [Discord](https://discord.gg/79ZZ66GH9E)! Rspack team and Rspack users are active there, and we're always looking for contributions.

## Links

| Name                                                                           | Description                                                                   |
| ------------------------------------------------------------------------------ | ----------------------------------------------------------------------------- |
| [awesome-rstack](https://github.com/rstackjs/awesome-rstack)                   | A curated list of awesome things related to Rstack                            |
| [agent-skills](https://github.com/rstackjs/agent-skills)                       | A collection of Agent Skills for Rstack                                       |
| [Rspack 2.x docs](https://v2.rspack.rs/)                                       | Documentation for Rspack 2.x (beta)                                           |
| [Rspack 1.x docs](https://rspack.rs/)                                          | Documentation for Rspack 1.x (latest)                                         |
| [Rspack 0.x docs](https://v0.rspack.rs/)                                       | Documentation for Rspack 0.x version                                          |
| [rspack-dev-server](https://github.com/web-infra-dev/rspack-dev-server)        | Dev server for Rspack                                                         |
| [rstack-examples](https://github.com/rstackjs/rstack-examples)                 | Examples showcasing Rstack                                                    |
| [rspack-sources](https://github.com/rstackjs/rspack-sources)                   | Rust port of [webpack-sources](https://www.npmjs.com/package/webpack-sources) |
| [rstack-design-resources](https://github.com/rstackjs/rstack-design-resources) | Design resources for Rstack                                                   |

## Contributors

<a href="https://github.com/web-infra-dev/rspack/graphs/contributors"><img src="https://opencollective.com/rspack/contributors.svg?width=890&button=false" /></a>

## Benchmark

See [Benchmark](https://ecosystem-benchmark.rspack.rs/).

## Credits

Thanks to:

- [The webpack team and community](https://webpack.js.org/) for creating a great bundler and ecosystem from which we draw a lot of inspiration.
- [@sokra](https://github.com/sokra) for the great work on the [webpack](https://github.com/webpack/webpack) project.
- [@ScriptedAlchemy](https://github.com/ScriptedAlchemy) for creating Module Federation and helping Rspack connect with the community.
- The [SWC](https://github.com/swc-project/swc) project created by [@kdy1](https://github.com/kdy1), which powers Rspack's code parsing, transformation and minification.
- The [esbuild](https://github.com/evanw/esbuild) project created by [@evanw](https://github.com/evanw), which inspired the concurrent architecture of Rspack.
- The [NAPI-RS](https://github.com/napi-rs/napi-rs) project created by [@Brooooooklyn](https://github.com/Brooooooklyn), which powers Rspack's node-binding implementation.
- The [Parcel](https://github.com/parcel-bundler/parcel) project created by [@devongovett](https://github.com/devongovett) which is the pioneer of rust bundler and inspired Rspack's incremental rebuild design.
- The [Vite](https://github.com/vitejs/vite) project created by [Evan You](https://github.com/yyx990803) which inspired Rspack's compatibility design of webpack's ecosystem.
- The `rolldown-legacy` project created by old Rolldown team, It's the predecessor of the [rolldown](https://github.com/rolldown) project, which explores the possibility of making a performant bundler in Rust with Rollup-compatible API. It inspires the design principles of Rspack.
- The [html-webpack-plugin](https://github.com/jantimon/html-webpack-plugin) project created by [@jantimon](https://github.com/jantimon), `@rspack/html-plugin` is a fork of [html-webpack-plugin](https://github.com/jantimon/html-webpack-plugin) to avoid some webpack API usage not supported in Rspack.
- The [Turbopack](https://github.com/vercel/turbo) project which inspired the AST path logic of Rspack.
- The [react-refresh-webpack-plugin](https://github.com/pmmmwh/react-refresh-webpack-plugin) created by [@pmmmwh](https://github.com/pmmmwh), which inspires implement [react refresh rspack plugin](https://github.com/rstackjs/rspack-plugin-react-refresh).
- The [prefresh](https://github.com/preactjs/prefresh) created by [@Jovi De Croock](https://github.com/JoviDeCroock), which inspires implement [preact refresh rspack plugin](https://github.com/rstackjs/rspack-plugin-preact-refresh).
- The [mini-css-extract-plugin](https://github.com/webpack/mini-css-extract-plugin) project created by [@sokra](https://github.com/sokra) which inspired implement css extract plugin.
- The [copy-webpack-plugin](https://github.com/webpack/copy-webpack-plugin) project created by [@kevlened](https://github.com/kevlened) which inspired implement copy rspack plugin.
- The [webpack-subresource-integrity](https://github.com/waysact/webpack-subresource-integrity) project created by [@jscheid](https://github.com/jscheid), which inspires implement subresource integrity rspack plugin.
- The [circular-dependency-plugin](https://github.com/aackerman/circular-dependency-plugin) project created by [@aackerman](https://github.com/aackerman), which inspres implement circular dependency rspack plugin.
- The [tracing-chrome](https://github.com/thoren-d/tracing-chrome) project created by [thoren-d](https://github.com/thoren-d), which inspires the implementation of Rspack tracing.

## License

Rspack is [MIT licensed](https://github.com/web-infra-dev/rspack/blob/main/LICENSE).
