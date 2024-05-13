> Based on _Webpack version: 5.73.0_.
> Some source code is omitted for cleaner demonstration in the example.

# Webpack Dependency

Explain how webpack dependency affects the compilation and what kind of problem that webpack was facing at the moment and the solution to the problem.

## Glossary

> What's the meaning of a word used to describe a feature?
>
> Why does the Webpack introduce this and what's the background of introducing this? What kind of problem Webpack was facing at the time?

### High-level presentations of _Dependencies_

- [Dependency(fileDependency)](https://webpack.js.org/api/loaders/#thisadddependency): An existing dependency that is marked as watchable. This is the widely-used type of dependency. CSS Preprocessors like `postcss` strongly depend on this in order to mark its dependency watchable.
- [ContextDependency](https://webpack.js.org/api/loaders/#thisaddcontextdependency): Most useful for requests in which Glob and Regexp were used. For real-world usage, see [[this](https://webpack.js.org/guides/dependency-management/#require-with-expression)](https://webpack.js.org/guides/dependency-management/#require-with-expression).
- [MissingDependency](https://webpack.js.org/api/loaders/#thisaddmissingdependency): A missing dependency to mark it watchable (handles the creation of files during compilation before watchers are attached correctly.)
- [BuildDependency](https://webpack.js.org/configuration/cache/#cachebuilddependencies): Related to persistent cache.
- PresentationalDependency: Dependencies that only affect presentation are mostly used with their associated template.

### Others

- [LoaderContext](https://webpack.js.org/api/loaders/#the-loader-context): Context provided by Webpack _loader-runner_, which can be accessed through `this` in each loader function.
- ModuleGraph: A graph to describe the relationship between modules.

## Guide-level explanation

### `Dependency`

`dependency`(`fileDependency`) stands for the file _dependency_ among `missingDependency` and `contextDependency`, etc. The created dependency will be marked as watchable, which is useful in _Hot Module Replacement_ in developer mode.

The implicit behavior for webpack internally in the case below is to create two dependencies internally.

```js
import foo from './foo';
import './style.css';
```

### `ContextDependency`

`contextDependency` is mostly used in scenarios where we want to dynamic load some module in runtime. In this case, webpack cannot assure which module it will be included in the final bundle at compile time. In order to make the code runnable in runtime, webpack has to firstly create multiple bundle modules corresponding to the matching filename such as `./components/a.js` and `./components/b.js`, etc.

```js
// index.js
import("./components" + componentName).then(...)
```

```js
// components/a.js
...
export default ComponentA;
```

```js
// components/b.js
...
export default ComponentB;
```

For loaders, you can access to `this.addContextDependency` in each loader function.
For plugins, you can access via `module.buildInfo.contextDependencies`.

## Reference-level explanation

> The abstraction of _Dependency_ of Webpack was introduced in Webpack version 0.9 with a big refactor. [Redirect to the commit](https://github.com/webpack/webpack/commit/ee01837d66a44f1dd52fd1e174a6669e0d18dd55)

### Stakeholders of _Dependency_

#### High-level

![image-20220919171608629](https://raw.githubusercontent.com/h-a-n-a/static/main/2022/09/upgit_20220919_1663578968.png)

#### Low-level

![image-20220919171841624](https://raw.githubusercontent.com/h-a-n-a/static/main/2022/09/upgit_20220919_1663579121.png)

### How _dependencies_ affect the creation of _module graph_?

#### Duplicated module detection

Each module will have its own `identifier`, for `NormalModule`, you can find this in `NormalModule#identifier`. If the identifier will be duplicated if inserted in `this._module`, then webpack will directly skip the remaining build process. [\[source\]](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L1270-L1274)

Basically, an `NormalModule` identifier contains these parts:

1. `type` \[`string`\]: The module type of a module. If the type of the module is `javascript/auto`, this field can be omitted
2. `request` \[`string`\]: Request to the module. All loaders whether it's inline or matched by a config will be stringified. If _inline match resource_ exists, inline loaders will be executed before any normal-loaders after pre-loaders. A module with a different loader passed through will be treated as a different module regardless of its path.
3. `layer`: applied if provided

#### Module resolution

`getResolve` is a loader API on the `LoaderContext`. Loader developers can pass `dependencyType` to its `option` which indicates the category of the module dependency that will be created. Values like `esm` can be passed, then webpack will use type `esm` to resolve the dependency.

The resolved dependencies are automatically added to the current module. This is driven by the internal plugin system of `enhanced-resolve`. Internally, `enhanced-resolve` uses plugins to handle the dependency registration like `FileExistsPlugin` [\[source\]](https://github.com/webpack/enhanced-resolve/blob/e5ff68aef5ab43b8197e864181eda3912957c526/lib/FileExistsPlugin.js#L34-L54) to detect whether a file is located on the file system or will add this file to a list of `missingDependency` and report in respect of the running mode of webpack. The collecting end of Webpack is generated by the `getResolveContext` in `NormalModule` [\[source\]](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/NormalModule.js#L513-L524)

#### _Module dependency_ in _ModuleGraph_

Here's a module graph with `esm` import between modules:

![image-20220919172119861](https://raw.githubusercontent.com/h-a-n-a/static/main/2022/09/upgit_20220919_1663579279.png)

The dependency type introduced by `import` or `require` is a derived dependency: _ModuleDependency_.

A _ModuleDependency_ contains three important fields.

1. `category`: used to describe the category of dependency. e.g. "esm" | "commonjs"
2. `request`: see the explanation above.
3. `userRequest`: Resource and its inline loader syntax will be stringified and applied, but loaders in `module.rules` will be omitted.

It's also good to note a field we will talk about later:

1. `assertions`: assertions in `import foo from "foo.json" assert { type: "json" }`

More fields can be found in abstract class of _Dependency_ and _ModuleDependency_. [source: Dependency](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Dependency.js#L88) [source: ModuleDependency](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/dependencies/ModuleDependency.js#L17)

```js
// null -> index.js

EntryDependency {
  category: "esm",
  request: "./index.js",
  type: "entry",
  _parentModule: undefined
}
```

```js
// index.js -> foo.js

HarmonyImportSideEffectDependency {
  category: "esm",
  request: "./foo",
  type: "harmony side effect evaluation",
  _parentModule: NormalModule { identifier: "index.js" }
}
```

```js
// index.js -> bar.js

HarmonyImportSideEffectDependency {
  category: "esm",
  request: "./bar",
  type: "harmony side effect evaluation",
  _parentModule: NormalModule { identifier: "index.js" }
}
```

```js
// bar.js -> foo.js
HarmonyImportSideEffectDependency {
  category: "esm",
  request: "./foo",
  type: "harmony side effect evaluation",
  _parentModule: NormalModule { identifier: "bar.js" }
}
```

#### Resolving a module

_ModuleDependencies_ with different dependency category such as `esm` or `commonjs` will affect the resolving part. For ECMAScript modules, they may prefer `"module"` to `"main"`, and for _CommonJS_ modules, they may use `"main"` in `package.json`. On top of that, conditional exports are also necessary to be taken into account. [doc](https://nodejs.org/api/packages.html#conditional-exports)

#### Different types of _module dependencies_

##### ESM-related derived types

There are a few of _ModuleDependencies_ introduced in ESM imports. A full list of each derived type can be reached at [\[source\]](https://github.com/webpack/webpack/blob/86a8bd9618c4677e94612ff7cbdf69affeba1268/lib/dependencies/HarmonyImportDependencyParserPlugin.js)

###### Import

**`HarmonyImportDependency`**

The basic type of harmony-related _module dependencies_ are below. [\[source\]](https://github.com/webpack/webpack/blob/86a8bd9618c4677e94612ff7cbdf69affeba1268/lib/dependencies/HarmonyImportDependency.js#L51)

**`HarmonyImportSideEffectDependency`**

```js
import { foo, bar } from './module';
import * as module from './module';
import foo from './module';
import './module';
```

Every import statement will come with a `HarmonyImportSideEffectDependency`, no matter how the specifiers look like. The specifier will be handled by `HarmonyImportSpecifierDependency` below.

The field `assertions` will be stored if any import assertions exist for later consumption.
The field `category` will be used as `dependencyType` to resolve modules.

**`HarmonyImportSpecifierDependency`**

```js
import { foo, bar } from './module';
import * as module from './module';
import foo from './module';
```

Example:

```js
import { foo, bar } from './module';

console.log(foo, bar);
```

Specifier will be mapped into a specifier dependency if and only if it is used. JavaScript parser will first tag each variable [\[source\]](https://github.com/webpack/webpack/blob/86a8bd9618c4677e94612ff7cbdf69affeba1268/lib/dependencies/HarmonyImportDependencyParserPlugin.js#L137), and then create corresponding dependencies on each reading of dependency. [\[source\]](https://github.com/webpack/webpack/blob/86a8bd9618c4677e94612ff7cbdf69affeba1268/lib/dependencies/HarmonyImportDependencyParserPlugin.js#L189) and finally be replaced to the generated `importVar`.

##### Export(They are not module dependencies to be actual, but I placed here for convenience)

**`HarmonyExportHeaderDependency`**

> PresentationalDependency

```js
export const foo = 'foo';
export default 'foo';
```

This is a _presentational dependency_. We will take more time on this later.

**`HarmonyExportSpecifierDependency`**

```js
export const foo = "foo"; // `foo` is a specifier

HarmonyExportSpecifierDependency {
  id: string;
  name: string;
}
```

**`HarmonyExportExpressionDependency`**

```js
export default "foo"; // "foo" is an expression

HarmonyExportExpressionDependency {
 range: [number, number] // range of the expression
 rangeStatement: [number, number] // range of the whole statement
}
```

## How _dependencies_ affect code generation

### _Presentational dependency_

> A type of dependency that only affects code presentation.

**`ConstDependency`**

```
ConstDependency {
  expression: string
  range: [number, number]
  runtimeRequirements: Set<string> | null
}
```

You can think of the passed `expression` as a `replacement` for the corresponding `range`. For the real world example, you can directly refer to _Constant Folding_.

### _Template_

Remember the fact that Webpack is an architecture wrapped around source code modifications. _Template_ is the solution that helps Webpack to do the real patch on the source code. Each dependency has its associated _template_ which affects a part of the code generation scoped per dependency. In other words, the effect of each _template_ is strictly scoped to its associated dependency.

![image-20220919173300220](https://raw.githubusercontent.com/h-a-n-a/static/main/2022/09/upgit_20220919_1663579980.png)

There are three types of modification:

- `source`
- `fragments`
- `runtimeRequirements`

A boilerplate of the dependency template looks like this:

```js
class SomeDependency {}

SomeDependency.Template = class SomeDependencyTemplate {
  /**
   * @param {Dependency} dependency the dependency for which the template should be applied
   * @param {ReplaceSource} source the current replace source which can be modified
   * @param {DependencyTemplateContext} templateContext the context object
   * @returns {void}
   */
  apply(dependency, source, templateContext) {
    // do code mod here
  }
};
```

There are three parameters in the function signature:

- dependency: The associated dependency of this template
- source: The source code represent in `ReplaceSource`, which can be used to replace a snippet of code with a new one, given the start and end position
- templateContext: A context of template, which stores the corresponding `module`, `InitFragments`, `moduleGraph`, `runtimeRequirements`, etc. (not important in this section)

#### `Source`

Again, given an example of [`ConstDependency`](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/dependencies/ConstDependency.js#L20), even if you don't have an idea what it is, it doesn't matter. We will cover this in the later sections.

The associated template modifies the code with `Source`(`ReplaceSource` to be more specific):

```js
ConstDependency.Template = class ConstDependencyTemplate extends (
  NullDependency.Template
) {
  apply(dependency, source, templateContext) {
    const dep = /** @type {ConstDependency} */ (dependency);

    // not necessary code is removed for clearer demonstration

    if (dep.runtimeRequirements) {
      for (const req of dep.runtimeRequirements) {
        templateContext.runtimeRequirements.add(req);
      }
    }

    source.replace(dep.range[0], dep.range[1] - 1, dep.expression);
  }
};
```

#### `runtimeRequirements`

As you can see from the `Source` section above, there is another modification we talked about: `runtimeRequirements`, It adds
runtime requirements for the current `compilation`. We will explain more in the later sections.

#### `Fragments`

Essentially, a [_fragment_](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/InitFragment.js) is a pair of code snippet that to be wrapped around each _module_ source. Note the wording "wrap", it could contain two parts `content` and `endContent` [\[source\]](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/InitFragment.js#L69). To make it more illustrative, see this:

<img width="390" alt="image" src="https://user-images.githubusercontent.com/10465670/190576169-43ac19c4-2783-46c3-9059-b64b1ff72c4e.png">

The order of the fragment comes from two parts:

1. The stage of a fragment: if the stage of two fragments is different, then it will be replaced corresponding to the order define by the stage
2. If two fragments share the same order, then it will be replaced in [position](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/InitFragment.js#L41) order.
   [\[source\]](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/InitFragment.js#L153-L159)

**A real-world example**

```js
import { foo } from './foo';

foo();
```

Given the example above, here's the code to generate a dependency that replaces `import` statement with `__webpack_require__`.

```js
// some code is omitted for cleaner demonstration
parser.hooks.import.tap(
  'HarmonyImportDependencyParserPlugin',
  (statement, source) => {
    const clearDep = new ConstDependency('', statement.range);
    clearDep.loc = statement.loc;
    parser.state.module.addPresentationalDependency(clearDep);

    const sideEffectDep = new HarmonyImportSideEffectDependency(source);
    sideEffectDep.loc = statement.loc;
    parser.state.module.addDependency(sideEffectDep);

    return true;
  },
);
```

Webpack will create two dependencies `ConstDependency` and `HarmonyImportSideEffectDependency` while parsing [\[source\]](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/dependencies/HarmonyImportDependencyParserPlugin.js#L110-L132).

Let me focus on `HarmonyImportSideEffectDependency` more, since it uses `Fragment` to do some patch.

```js
// some code is omitted for cleaner demonstration
HarmonyImportSideEffectDependency.Template = class HarmonyImportSideEffectDependencyTemplate extends (
  HarmonyImportDependency.Template
) {
  apply(dependency, source, templateContext) {
    super.apply(dependency, source, templateContext);
  }
};
```

As you can see in its associated _template_ [\[source\]](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/dependencies/HarmonyImportSideEffectDependency.js#L59), the modification to the code is made via its superclass `HarmonyImportDependency.Template` [\[source\]](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/dependencies/HarmonyImportDependency.js#L244).

```js
// some code is omitted for cleaner demonstration
HarmonyImportDependency.Template = class HarmonyImportDependencyTemplate extends (
  ModuleDependency.Template
) {
  apply(dependency, source, templateContext) {
    const dep = /** @type {HarmonyImportDependency} */ (dependency);
    const { module, chunkGraph, moduleGraph, runtime } = templateContext;

    const referencedModule = connection && connection.module;

    const moduleKey = referencedModule
      ? referencedModule.identifier()
      : dep.request;
    const key = `harmony import ${moduleKey}`;

    // 1
    const importStatement = dep.getImportStatement(false, templateContext);
    // 2
    templateContext.initFragments.push(
      new ConditionalInitFragment(
        importStatement[0] + importStatement[1],
        InitFragment.STAGE_HARMONY_IMPORTS,
        dep.sourceOrder,
        key,
        // omitted for cleaner code
      ),
    );
  }
};
```

As you can see from the simplified source code above, the actual patch made to the generated code is via `templateContext.initFragments`(2). The import statement generated from dependency looks like this.

```js
/* harmony import */ var _foo__WEBPACK_IMPORTED_MODULE_0__ =
  __webpack_require__(/*! ./foo */ './src/foo.js'); //(1)
```

Note, the real require statement is generated via _initFragments_, `ConditionalInitFragment` to be specific. Don't be afraid of the naming, for more information you can see the [background](https://github.com/webpack/webpack/pull/11802) of this _fragment_, which let's webpack to change it from `InitFragment` to `ConditionalInitFragment`.

**How does webpack solve the compatibility issue?**

For ESM modules, webpack will additionally call a helper to define `_esModule` on exports as an hint:

```js
__webpack_require__.r(__webpack_exports__);
```

The call of a helper is always placed ahead of any `require` statements. Probably you have already get this as the stage of `STAGE_HARMONY_EXPORTS` has high priority than `STAGE_HARMONY_IMPORTS`. Again, this is achieved via `initFragments`. The logic of the compatibility helper is defined in [this](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/dependencies/HarmonyCompatibilityDependency.js) file, you can check it out.

### Runtime

Runtime generation is based on the previously collected `runtimeRequirements` in different dependency templates and is done after the code generation of each module. Note: it's not after the `renderManifest`, but it's after the code generation of each module.

![image-20220919173829765](https://raw.githubusercontent.com/h-a-n-a/static/main/2022/09/upgit_20220919_1663580309.png)In the first iteration of collection, Sets of `runtimeRequirements` are collected from the module's code generation results and added to each `ChunkGraphModule`.

In the second iteration of collection, the collected `runtimeRequirements` are already stored in `ChunkGraphModule`, so Webpack again collects them from there and stores the runtimes required by each chunk of `ChunkGraphChunk`. It's kind of the hoisting procedure of the required runtimes.

Finally, also known as the third iteration of collection, Webpack hoists `runtimeRequirements` from those chunks that are referenced by the entry chunk and get it hoisted on the `ChunkGraphChunk` using a different field named `runtimeRequirementsInTree` which indicates not only does it contains the runtime requirements by the chunk but also it's children runtime requirements.

![image-20220919174132772](https://raw.githubusercontent.com/h-a-n-a/static/main/2022/09/upgit_20220919_1663580492.png)

The referenced source code you can be found it [here](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L3379) and these steps are basically done in `processRuntimeRequirements`. This let me recall the linking procedure of a rollup-like bundler. Anyway, after this procedure, we can finally generate _runtime modules_. Actually, I lied here, huge thanks to the hook system of Webpack, the creation of _runtime modules_ is done in this method via calls to `runtimeRequirementInTree`[\[source\]](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L3498). No doubt, this is all done in the `seal` step. After that, webpack will process each chunk and create a few code generation jobs, and finally, emit assets.

### _Hot module replacement_

Changes made via _hot module replacement_ is mostly come from `HotModuleReplacementPlugin`.

Given the code below:

```js
if (module.hot) {
  module.hot.accept(...)
}
```

Webpack will replace expressions like `module.hot` and `module.hot.accept`, etc with `ConstDependency` as the _presentationalDependency_ as I previously talked about. [\[source\]](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/HotModuleReplacementPlugin.js#L97-L101)

With the help of a simple expression replacement is not enough, the plugin also introduce additional runtime modules for each entries. [\[source\]](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/HotModuleReplacementPlugin.js#L736-L748)

The plugin is quite complicated, and you should definitely checkout what it actually does, but for things related to dependency, it's enough.

## How _dependencies_ affect production optimizations

### Constant folding

> The logic is defined in ConstPlugin : [\[source\]](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/ConstPlugin.js#L135)

_Constant folding_ is a technique that used as an optimization for optimization. For example:

**Source**

```js
if (process.env.NODE_ENV === "development") {
   ...
} else {
   ...
}
```

**Generated**

```js
if (true) {
  ...
}
```

With mode set to `"development"`, webpack will "fold" the expression `process.env.NODE_ENV === "development"` into an expression of `"true"` as you can see for the code generation result.

In the `make` procedure of webpack, Webpack internally uses an `JavaScriptParser` for JavaScript parsing. If an `ifStatement` is encountered, Webpack creates a corresponding `ConstDependency`. Essentially, for the `ifStatement`, the `ConstDependency` looks like this :

```js
ConstDependency {
  expression: "true",
  range: [start, end] // range to replace
}
```

It's almost the same with `else` branch, if there is no _side effects_(refer to source code for more detail), Webpack will create another `ConstDependency` with `expression` set to `""`, which in the end removes the `else` branch.

In the `seal` procedure of Webpack, the record of the dependency will be applied to the original source code and generate the final result as you may have already seen above.

### Tree shaking & DCE

Tree-shaking is a technique of a bundle-wise DCE(dead code elimination). In the following content, I will use tree-shaking as a wording for bundle-wise and DCE for module-wise code elimination. (I know it's not quite appropriate, but you get the point)

Here's an example:

```js
// webpack configuration
module.exports = {
  optimization: {
    usedExports: true,
  },
};
```

![image-20220919182656468](https://raw.githubusercontent.com/h-a-n-a/static/main/2022/09/upgit_20220919_1663583216.png)

![image-20220919190553215](https://raw.githubusercontent.com/h-a-n-a/static/main/2022/09/upgit_20220919_1663585553.png)

![image-20220919190925073](https://raw.githubusercontent.com/h-a-n-a/static/main/2022/09/upgit_20220919_1663585765.png)

As you can see from the red square, the `initFragment` is generated based on the usage of the exported symbol in the `HarmonyExportSpecifierDependency` [\[source\]](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/dependencies/HarmonyExportSpecifierDependency.js#L91-L107)

If `foo` is used in the graph, then the generated result will be this:

```js
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
  /* harmony export */ foo: function () {
    return /* binding */ foo;
  },
  /* harmony export */
});
const foo = 'foo';
```

In the example above, the `foo` is not used, so it will be excluded in the code generation of the template of `HarmonyExportSpecifierDependency` and it will be dead-code-eliminated in later steps. For terser plugin, it eliminates all unreachable code in `processAssets` [\[source\]](https://github.com/webpack-contrib/terser-webpack-plugin/blob/580f59c5d223a31c4a9c658a6f9bb1e59b3defa6/src/index.js#L836).

## Things related to Persistent cache

_TODO_

## Wrap it up!

Let's wrap everything up in a simple example! Isn't it exciting?

![image-20220919223228146](https://raw.githubusercontent.com/h-a-n-a/static/main/2022/09/upgit_20220919_1663597948.png)

Given a module graph that contains three modules, the entry point of this bundle is `index.js`. To not make this example too complicated, we use normal import statements to reference each module (i.e: only one chunk that bundles everything will be created).

### `Make`

![image-20220919223558327](https://raw.githubusercontent.com/h-a-n-a/static/main/2022/09/upgit_20220919_1663598158.png)

### Dependencies after `make`

![image-20220919223720739](https://raw.githubusercontent.com/h-a-n-a/static/main/2022/09/upgit_20220919_1663598240.png)

### `seal`

![image-20220920180915326](https://raw.githubusercontent.com/h-a-n-a/static/main/2022/09/upgit_20220920_1663668558.png)
