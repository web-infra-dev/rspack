# Rspack loader

## Related PRs

- [rspack#2780](https://github.com/web-infra-dev/rspack/pull/2789)
- [rspack#2808](https://github.com/web-infra-dev/rspack/pull/2808)

The old architecture is a quite simple version, which only supports loaders for normal stage.
Pitching loader does not put into consideration. The basic concept of the old version is to
convert the normal loader to a native function which can be called from the Rust side.
Furthermore, for performance reason, Rspack also composes loaders from the JS side to
mitigate the performance issue of Node/Rust communications.

In this new architecture, loaders will not be converted directly into native functions.
Instead, it is almost the same with how webpack's loader-runner resolves its loaders, by
leveraging the identifier. Every time Rspack wants to invoke a JS loader, the identifiers will
be passed to the handler passed by Node side to process. The implementation also keeps
the feature of composing JS loaders for performance reason.

## Guide-level explanation

The refactor does not introduce any other breaking changes. So it's backwards compatible.
The change of the architecture also help us to implement pitching loader with composability.

### Pitching loader

Pitching loader is a technique to change the loader pipeline flow. It is usually used with
inline loader syntax for creating another loader pipeline. style-loader, etc and other loaders
which might consume the evaluated result of the following loaders may use this technique.
There are other technique to achieve the same ability, but it's out of this article's topic.

See [Pitching loader](https://webpack.js.org/api/loaders/#pitching-loader) for more detail.

## Reference-level explanation

### Actor of loader execution

In the original implementation of loader, Rspack will convert the normal loaders in the first place,
then pass it to the Rust side. In the procedure of building modules, these loaders will be called directly:

![Old architecture](https://user-images.githubusercontent.com/10465670/233357319-e80f6b32-331c-416d-b4b5-30f3e0e394bd.png)

The loader runner is only on the Rust side and execute the loaders directly from the Rust side.
This mechanism has a strong limit for us to use webpack's loader-runner for composed loaders.

In the new architecture, we will delegate the loader request from the Rust core to a dispatcher
located on the JS side. The dispatcher will normalize the loader and execute these using a modified
version of webpack's loader-runner:

![image](https://user-images.githubusercontent.com/10465670/233357805-923e0a27-609d-409a-b38d-96a083613235.png)

Loader functions for pitch or normal will not be passed to the Rust side. Instead, each JS loader has
its identifier to uniquely represent each one. If a module requests a loader for processing the module,
Rspack will pass identifier with options to the JS side to instruct the Webpack like loader-runner to
process the transform. This also reduces the complexity of writing our own loader composer.

### Passing options

Options will normally be converted to query, but some of the options contain fields that cannot be
serialized, Rspack will reuse the _**loader ident**_ created by webpack to uniquely identify the option
and restore it in later loading process.

### Optimization for pitching

As we had known before, each loader has two steps, pitch and normal. For a performance friendly
interoperability, we must reduce the communication between Rust and JS as minimum as possible.
Normally, the execution steps of loaders will look like this:

![image](https://user-images.githubusercontent.com/10465670/233360942-7517f22e-3861-47cb-be9e-6dd5f5e02a4a.png)

The execution order of the loaders above will looks like this:

```
loader-A(pitch)
   loader-B(pitch)
      loader-C(pitch)
   loader-B(normal)
loader-A(normal)
```

The example above does not contain any JS loaders, but if, say, we mark these loaders registered on the
JS side:

![image](https://user-images.githubusercontent.com/10465670/233362338-93e922f6-8812-4ca9-9d80-cf294e4f2ff8.png)

The execution order will not change, but Rspack will compose the step 2/3/4 together for only a single
round communication.
