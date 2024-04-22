<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://lf3-static.bytednsdoc.com/obj/eden-cn/dhozeh7vhpebvog/rspack-banner-1610-dark.png">
  <img alt="Rspack Banner" src="https://lf3-static.bytednsdoc.com/obj/eden-cn/dhozeh7vhpebvog/rspack-banner-1610.png">
</picture>

<h2 align="center">A fast Rust-based web bundler</h2>

<p align="center">
  <a href="https://discord.gg/79ZZ66GH9E">
    <img src="https://img.shields.io/discord/977448667919286283?logo=discord&label=discord&colorB=EDED91" alt="discord channel" />
  </a>
  <a href="https://www.npmjs.com/package/@rspack/core?activeTab=readme">
   <img src="https://img.shields.io/npm/v/@rspack/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" />
  </a>
  <a href="https://npmcharts.com/compare/@rspack/core?minimal=true">
    <img src="https://img.shields.io/npm/dm/@rspack/core.svg?style=flat-square&colorA=564341&colorB=EDED91" alt="downloads" />
  </a>
  <a href="https://github.com/web-infra-dev/rspack/blob/main/LICENSE">
    <img src="https://img.shields.io/npm/l/@rspack/core?style=flat-square&colorA=564341&colorB=EDED91" alt="license" />
  </a>
</p>

English | [简体中文](./README.zh-CN.md)

## ✨ Features

- 🚀 **Fast Startup**: Based on Rust, the build speed is extremely fast, bringing you the ultimate development experience.
- ⚡ **Lightning HMR**: With a built-in incremental compilation mechanism, HMR is extremely fast and fully capable of developing large-scale projects.
- 📦 **Webpack Interoperable**: Compatible with the architecture and ecosystem of webpack, no need to build the ecosystem from scratch.
- 🎨 **Batteries Included**: Out-of-the-box support for TypeScript, JSX, CSS, CSS Modules, Sass, and more.
- 🛠️ **Production Optimization**: Various optimization strategies are built in by default, such as tree shaking, minification, etc.
- 🎯 **Framework Agnostic**: Not bound to any frontend framework, ensuring enough flexibility.

Read [Introduction](https://rspack.dev/guide/introduction.html) for details.

## Getting Started

- [Quick Start](https://rspack.dev/guide/quick-start.html)

## Contribution

Please read the [contributing guide](./CONTRIBUTING.md) and let's build Rspack together.

### Code of Conduct

This repo has adopted the ByteDance Open Source Code of Conduct. Please check [Code of Conduct](./CODE_OF_CONDUCT.md) for more details.

## Community

Come chat with us on [Discord](https://discord.gg/79ZZ66GH9E)! Rspack team and Rspack users are active there, and we're always looking for contributions.

## Links

| Name                                                                                     | Description                                                                   |
| ---------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| [Rspack website](https://rspack.dev)                                                     | Official documentation for Rspack                                             |
| [rspack-examples](https://github.com/rspack-contrib/rspack-examples)                     | Rspack configuration examples                                                 |
| [rspack-sources](https://github.com/web-infra-dev/rspack-sources)                        | Rust port of [webpack-sources](https://www.npmjs.com/package/webpack-sources) |
| [rspack-migration-showcase](https://github.com/web-infra-dev/rspack-migration-showcase)  | Migration showcases for Rspack                                                |
| [rspack-compat](https://github.com/web-infra-dev/rspack-compat)                          | Rspack compatible loaders and plugins examples                                |
| [rsfamily-design-resources](https://github.com/rspack-contrib/rsfamily-design-resources) | Design resources for Rspack, Rsbuild, Rspress and Rsdoctor                    |

## Contributors

<a href="https://github.com/web-infra-dev/rspack/graphs/contributors"><img src="https://opencollective.com/rspack/contributors.svg?width=890&button=false" /></a>

## Benchmark

See [Benchmark](https://web-infra-dev.github.io/rspack-ecosystem-benchmark/)

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
- The [rolldown-legacy](https://github.com/rolldown-rs/rolldown-legacy) project created by old [Rolldown team](https://github.com/rolldown-rs), It's the predecessor of the [rolldown](https://github.com/rolldown) project, which explores the possibility of making a performant bundler in Rust with Rollup-compatible API. It inspires the design principles of Rspack.
- The [html-webpack-plugin](https://github.com/jantimon/html-webpack-plugin) project created by [@jantimon](https://github.com/jantimon), `@rspack/html-plugin` is a fork of [html-webpack-plugin](https://github.com/jantimon/html-webpack-plugin) to avoid some webpack API usage not supported in Rspack.
- The [Turbopack](https://github.com/vercel/turbo) project which inspired the AST path logic of Rspack.
- The [react-refresh-webpack-plugin](https://github.com/pmmmwh/react-refresh-webpack-plugin) created by [@pmmmwh](https://github.com/pmmmwh), which inspires implement react refresh.

## License

Rspack is [MIT licensed](https://github.com/web-infra-dev/rspack/blob/main/LICENSE).

