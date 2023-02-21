# M

## Module

`Module` is an abstraction of a file, which could be Javascript, CSS, or any other type of file. It's holds the information that plugin care about. The bundler does not know the detail of `Module`s and only intereact with it via the [`Module` trait](https://github.com/modern-js-dev/rspack/blob/e55f029498d965178e36dc0882c79b76e5883bfe/crates/rspack_core/src/module.rs#L72-L94).

## ModuleGraphModule

`ModuleGraphModule` is wrapper of `Module`. It holds the `Module` object and store some extra information that bundler care about.

## ModuleType

`ModuleType` is an enum that represents the type of a `Module`. When rendering a module, the module will be rendered based on each `SourceType` request. In other words, `ModuleType` does not strongly related to the actual code generation result. For types supported, you may refer to [this](https://github.com/modern-js-dev/rspack/blob/3d981eea519f36fe0e53cdb878dab447a4e70cc8/crates/rspack_core/src/lib.rs#L37)

# S

## SourceType

`SourceType` is used to describe the type of a code snippet. Each `Module` may contain different `SourceType`. E.g. an `Asset Module` may contain `SourceType::Javascript` and `SourceType::Asset`, which means it has the ability to render both types. The bundler will use the `SourceType` as a part of the request to request a render from a `Module`. See [this](https://github.com/modern-js-dev/rspack/blob/3d981eea519f36fe0e53cdb878dab447a4e70cc8/crates/rspack_core/src/lib.rs#L30) to get the full list of supported `SourceType`.
