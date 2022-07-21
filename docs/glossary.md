# M

## Module

`Module` is an abstraction of a file, which could be javascript, CSS, or any other type of file. It's holds the informations that plugin care about. The bundler does not know the detail of `Module`s and only intereact with it via the [`Module` trait](https://github.com/speedy-js/rspack/blob/e55f029498d965178e36dc0882c79b76e5883bfe/crates/rspack_core/src/module.rs#L72-L94).

## ModuleGraphModule

`ModuleGraphModule` is wrapper of `Module`. It holds the `Module` object and store some extra informations that bundler care about.
