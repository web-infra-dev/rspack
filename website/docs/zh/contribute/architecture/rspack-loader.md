# Rspack loader

## 相关的 PRs

- [rspack#2780](https://github.com/web-infra-dev/rspack/pull/2789)
- [rspack#2808](https://github.com/web-infra-dev/rspack/pull/2808)

旧的架构是一个非常简单的版本，仅支持正常阶段的 loader。不考虑 pitch 的 loader。
旧版本的基本概念是将普通 loader 转换为可以从 Rust 端调用的 native 函数。
此外，出于性能考虑，Rspack 还从 JS 端编写了 loader 来缓解 Node/Rust 通信的性能问题。

在这个新架构中，loader 不会直接转换为 native 函数。相反，它与 webpack 的 loader-runner 解析其 loader 的方式几乎相同，通过利用标识符(identifier)。
每次 Rspack 想要调用 JS loader 时，标识符都会被传递给 Node 侧的 handler 进行处理。这个实现还保留了由于性能问题编写 JS loader 的特性。

## 概述级解释

重构不会引入任何其他重大更改。所以它是向后兼容的。
架构的改变也帮助我们实现了具有可组合性的 Pitch loader。

### Pitching loader

Pitching loader 是一种改变 loader pipeline 流程的技术。
它通常和内联 loader 的语法一起使用来创建另一个 loader pipeline。
style-loader 等其他可能会消费以下 loader 的处理结果的 loader 可能会使用此技术。
还有其他技术可以实现相同的功能，但这不属于本文的主题。

查看 [Pitching loader](https://webpack.js.org/api/loaders/#pitching-loader) 来获得更多信息。

## 参考级解释

### loader 执行的操作

在 loader 的原始实现中，Rspack首先会转换普通的 loader，
然后将其传递给 Rust 端。在构建模块的过程中，将直接调用这些 loader：

![旧架构](https://user-images.githubusercontent.com/10465670/233357319-e80f6b32-331c-416d-b4b5-30f3e0e394bd.png)

loader 运行程序仅位于 Rust 端，并直接从 Rust 端执行加载器。
这种机制对于我们使用 webpack 的 loader-runner 来组合 loader 有很大的限制。

在新架构中，我们将把来自 Rust 核心的 loader 请求委托给位于 JS 端的调度程序。
调度程序将规范化加载程序并使用修改后的代码执行这些加载程序
webpack 的 loader-runner 版本：

![image](https://user-images.githubusercontent.com/10465670/233357805-923e0a27-609d-409a-b38d-96a083613235.png)

normal 或者 pitch 的 loader 函数不会传递到 Rust 端。相反，每个 JS 加载器都有
它的唯一的标识符。如果一个模块请求 loader 来处理该模块，
Rspack 会将带有选项的标识符传递给 JS 端，以指示 Webpack（如 loader-runner）
处理转换。这也降低了编写我们自己的 loader 编译的复杂度。

### 传递 options 选项

选项通常会转换为 query，但有些选项包含无法序列化的字段，Rspack 会复用 webpack 创建的**_loader ident_** 作为唯一标识选项
并在以后的加载过程中恢复它。

### 针对 pitching 的优化

正如我们之前所知，每个 loader 都有两个步骤：pitch 和 normal。

对于性能友好的互操作性，我们必须尽可能减少 Rust 和 JS 之间的通信。

通常情况下，loader 的执行步骤会是这样的：

![image](https://user-images.githubusercontent.com/10465670/233360942-7517f22e-3861-47cb-be9e-6dd5f5e02a4a.png)

上面的加载器的执行顺序如下所示：

```
loader-A(pitch)
   loader-B(pitch)
      loader-C(pitch)
   loader-B(normal)
loader-A(normal)
```

上面的例子不包含任何 JS loader，但是如果我们将这些 loader 标记为注册在 JS 端：

![image](https://user-images.githubusercontent.com/10465670/233362338-93e922f6-8812-4ca9-9d80-cf294e4f2ff8.png)

执行顺序不会改变，但 Rspack 会将步骤 2/3/4 组合在一起，仅执行一次循环通讯。
