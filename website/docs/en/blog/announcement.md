---
date: 2023-03-06 21:37:00
---

# Announcing Rspack

> March 06, 2023

Today we are so thrilled to announce that Rspack is officially released! Rspack is a Rust-based JavaScript bundler developed by the ByteDance Web Infra team that has features including high-performance, webpack interoperability, flexible configuration etc. Rspack has solved many problems in our scenarios and improved the developer experience for JavaScript engineers. To help more people get involved in this exciting project, we decided to open source this project. You are welcomed to create a pull request or issue.

## Why Rspack?

There are a lot of giant JavaScript applications inside ByteDance. They have very complex build configurations/scripts which may cost ten minutes to half an hour. We have tried so many ways to improve the build performance, but all existing solutions in the world lead to some other issues while solving some of them. After tons of work, we understand the requirements for a bundler are:

- Dev startup performance. `npm run dev` is a daily script for developers that may run many times. Reducing the cost of them to one minute from ten minutes is really life-saving.
- Build performance. `npm run build` is common in CI/CD environments and determines the efficiency of launch. Many giant applications in ByteDance are built in 20 ~ 30 minutes. If we can reduce it to 3~5 minutes, developers will be really productive.
- Flexible configuration. Giant projects always have complex configurations and can't be standardized. Back in time, we migrated some of the projects to other build tools to improve build performance, and the hardest part is changing the configuration.
- Production optimization. We tried various solutions in the community and webpack gave the best result in production optimization like chunk-splitting, tree-shaking, etc. A better chunk strategy can help web apps get better metrics performance.

In conclusion, we decided to build our own bundler, which is `Rspack`.

## How is Rspack doing now?

The Rspack project started about 11 months ago. Although it's still in the early stages, it can bring 5~10 times improvement to applications' build scripts. The metrics can be better when we finish all the optimizations.

Rspack has completed the architecture of webpack loader. It means you can use all kinds of loaders in the community, such as `babel-loader`, `less-loader`, `svgr` etc. We are planning to support all features of loader in Rspack. By that time, you can use loaders which haven't been supported for now, such as `vue-loader`.

Rspack currently only supports memory cache. Persistent and portable cache will be added in the future. We are working on a build system that can make cache shareable between two devices or environments. And Rspack will help accomplish that.

Rspack is now available in all frameworks inside ByteDance, and we are trying to collaborate with all friends in the community. Just like webpack, Rspack is an infrastructure for JavaScript ecosystems, which means that frameworks and Rspack can be beneficial for each other.

## Acknowledgement

Rspack can not be shipped today without the inspiration and support of various projects in the community. We would like to show our respect to these predecessors:

- [The webpack team and community](https://webpack.js.org/) for creating a great bundler and ecosystem from which we draw a lot of inspiration.
- [@sokra](https://github.com/sokra) for the great work on the [webpack](https://github.com/webpack/webpack) project.
- [@ScriptedAlchemy](https://github.com/ScriptedAlchemy) for creating Module Federation and helping Rspack connect with the community.
- The [SWC](https://github.com/swc-project/swc) project created by [@kdy1](https://github.com/kdy1), which powers Rspack's code parsing, transformation and minification.
- The [esbuild](https://github.com/evanw/esbuild) project created by [@evanw](https://github.com/evanw), which inspired the concurrent architecture of Rspack.
- The [NAPI-RS](https://github.com/napi-rs/napi-rs) project created by [@Brooooooklyn](https://github.com/Brooooooklyn), which powers Rspack's node-binding implementation.
- The [Parcel](https://github.com/parcel-bundler/parcel) project created by [@devongovett](https://github.com/devongovett) which is the pioneer of rust bundler and inspired Rspack's incremental rebuild design.
- The [Vite](https://github.com/vitejs/vite) project created by [Evan You](https://github.com/yyx990803) which inspired Rspack's compatibility design of webpack's ecosystem.
- The [Rolldown](https://github.com/rolldown-rs/rolldown) project created by [Rolldown team](https://github.com/sponsors/rolldown-rs), which explores the possibility of making a performant bundler in Rust with Rollup-compatible API. It inspires the design principles of Rspack.
- The [html-webpack-plugin](https://github.com/jantimon/html-webpack-plugin) project created by [@jantimon](https://github.com/jantimon), `@rspack/html-plugin` is a fork of [html-webpack-plugin](https://github.com/jantimon/html-webpack-plugin) to avoid some webpack API usage not supported in Rspack.
- The [Turbopack](https://github.com/vercel/turbo) project which inspired the ast path logic of Rspack.

## Future plans

### Improve basic capabilities

Keep building Rspack will be our top priority. Compared with webpack, Rspack is still a baby, lacking complex features. Please keep sending us feedback on feature requests. We will finish them step by step.

### Working with community partners

We would love to offer some help with Rspack integration in your framework. If you are an engineer maintaining a framework who happens to be interested in giving Rspack a try, please contact us.
We have also established a partnership with the webpack team. Rspack is an attempt to optimize webpack performance using Rust, and in the future, we will explore more possibilities for optimizing webpack together with the webpack team. When Rspack reaches a certain level of maturity, webpack will attempt to integrate Rspack into webpack with experiments flag.

### Improve plugin capabilities

Rspack has supported most of the loader APIs, but only a few plugin APIs. There are two reasons why we haven't supported them all. One is that some APIs are bad for performance, so we didn't explore them for developers. And the other reason is simply lack of time, so you can create a merge request to help us.

A high performance plugin system is under discussion. It may be shipped out someday. Hopefully it can help developers get shorter build time while accessing a flexible configuration.

### Continuously improve performance

Currently, Rspack is a project with performance as the core selling point, so in the future we will do a lot of things to maintain this feature, such as improving the performance observation lab and doing a good job of performance prevention; using concurrent/multi-core friendly algorithms in more scenarios; developing a caching system that can be shared across platforms; optimizing memory usage and consumption, etc.

### Build a quality assurance system

Webpack has already accumulated very rich test cases, and in the future Rspack will reuse the existing test cases of Webpack to improve its code coverage. Build a better CI system, and build an Ecosystem CI system with community projects to ensure that project upgrades do not cause breaks on upstream projects, to ensure long-term project health, and to ensure long-term increase in test coverage.

## Trial

- Quick start: [rspack.dev](https://rspack.dev)
- GitHub Repo: [github.com/web-infra-dev/rspack](https://github.com/web-infra-dev/rspack)
