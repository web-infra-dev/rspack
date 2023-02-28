<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://lf3-static.bytednsdoc.com/obj/eden-cn/rjhwzy/ljhwZthlaukjlkulzlp/rspack-banner-1610-dark.png">
  <img alt="Rspack Banner" src="https://lf3-static.bytednsdoc.com/obj/eden-cn/rjhwzy/ljhwZthlaukjlkulzlp/rspack-banner-1610.png">
</picture>

<h2 align="center">A fast Rust-based web bundler</h2>

<p align="center">
  <img src="https://img.shields.io/npm/v/@rspack/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" />
  <img src="https://img.shields.io/npm/dm/@rspack/core.svg?style=flat-square&colorA=564341&colorB=EDED91" alt="downloads" />
  <a href="https://github.com/modern-js-dev/rspack/blob/main/LICENSE"><img src="https://img.shields.io/npm/l/@rspack/core?style=flat-square&colorA=564341&colorB=EDED91" alt="license" /></a>
</p>

English | [简体中文](./README.zh-CN.md)

## ✨ Features

- 🚀 **Fast**: Based on Rust, the build speed is extremely fast, bringing you the ultimate development experience.
- 📦 **Webpack Interoperable**: Targeted interoperability the Webpack ecosystem, no need to build your ecosystem from scratch.
- 🎨 **Batteries Included**: Out-of-the-box support for Typescript, JSX, CSS, CSS Modules, Sass, and more.

Read [Introduction](https://rspack.org/guide/introduction.html) for details.

## Getting Started

- [Quick Start](https://rspack.org/guide/getting-started.html)

## Contribution

> **Rspack is in active development and still missing lots of webpack APIs and and some APIs may not be stable at the moment, we are working on implementing these APIs and keep improving the stability of Rspack, if you have any suggestions or comments, please feel free to submit a PR or Issue, we would appreciate it.**

Please read the [contributing guide](./CONTRIBUTING.md) and let's build Rspack together.

### Code of Conduct

This repo has adopted the Bytedance Open Source Code of Conduct. Please check [Code of Conduct](./CODE_OF_CONDUCT.md) for more details.

## Links

| Name                                                                                    | Description                                                                 |
| --------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| [rspack website](https://rspack.org)                                                    | Official documentation for rspack                                           |
| [rspack website repo](https://github.com/modern-js-dev/rspack-website)                  | Repository of official documentation                                        |
| [rspack-sources](https://github.com/modern-js-dev/rspack-sources)                       | Rusty [webpack-sources](https://www.npmjs.com/package/webpack-sources) port |
| [rspack-migration-showcase](https://github.com/modern-js-dev/rspack-migration-showcase) | Migration showcases for rspack                                              |

## Credits

Thanks to:

- [The webpack team and community](https://webpack.js.org/) for creating a great bundler and ecosystem from which we draw a lot of inspiration.
- [@sokra](https://github.com/sokra) for the great work on the [webpack](https://github.com/webpack/webpack) project.
- [@ScriptedAlchemy](https://github.com/ScriptedAlchemy) for creating Module Federation and helping Rspack connect with the community.
- The [SWC](https://github.com/swc-project/swc) project created by [@kdy1](https://github.com/kdy1), which powers Rspack's code compilation and minification.
- The [esbuild](https://github.com/evanw/esbuild) project created by [@evanw](https://github.com/evanw), which inspired the concurrent architecture of Rspack.
- The [NAPI-RS](https://github.com/napi-rs/napi-rs) project created by [@Brooooooklyn](https://github.com/Brooooooklyn), which powers Rspack's node-binding implementation.
- The [Parcel](https://github.com/parcel-bundler/parcel) project created by [@devongovett](https://github.com/devongovett) which inspired Rspack's incremental rebuild design.
- The [Vite](https://github.com/vitejs/vite) project created by [Evan You](https://github.com/yyx990803) which inspired Rspack's compatibility design of webpack's ecosystem.
- The [Rolldown](https://github.com/rolldown-rs/rolldown) project created by [Rolldown team](https://github.com/sponsors/rolldown-rs), which explores the possibility of making a performant bundler in Rust with Rollup-compatible API. It inspires the design principles of Rspack.
- The [html-webpack-plugin](https://github.com/jantimon/html-webpack-plugin) project created by [@jantimon](https://github.com/jantimon), which inspired `@rspack/html-plugin`.

## License

Rspack is [MIT licensed](https://github.com/modern-js-dev/rspack/blob/main/LICENSE).
