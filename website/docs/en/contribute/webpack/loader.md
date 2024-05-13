> Based on _Webpack version: 5.73.0_.

# Loader

Explain how webpack loader works. Even though it's a little bit long and tedious, It's still a teeny-tiny peek at the loader system of Webpack.

# Glossary

> What's the meaning of a word used to describe a feature?
>
> Why does the Webpack introduce this and what's the background of introducing this? What kind of problem Webpack was facing at the time?

## Request Related

```javascript
import Styles from '!style-loader!css-loader?modules!./styles.css';
```

- [Inline loader syntax](https://webpack.js.org/concepts/loaders/#inline): The syntax that chains the loader together within the specifier, followed by the file requested. e.g. `!style-loader!css-loader?modules!./style.css`
- `request`: The request with _inline loader syntax_ retained. Webpack will convert relative URLs and module requests to absolute URLs for loaders and files requested. e.g. `!full-path-to-the-loader-separated-with-exclamation-mark!full-path-to-styles.css`

## Resource Related

```javascript
import foo from './index.js?vue=true&style#some-fragment';
```

- [`resource`](https://webpack.js.org/api/loaders/#thisresource): The absolute path to the requested file with `query` and `fragment` retained but inline loader syntax removed. e.g. `absolute-path-to-index-js.js?vue=true&style#some-fragment`
- [`resourcePath`](https://webpack.js.org/api/loaders/#thisresourcepath): The absolute path to the requested file only. e.g. `absolute-path-to-index-js.js`
- [`resourceQuery`](https://webpack.js.org/api/loaders/#thisresourcequery): Query with question mark `?` included. e.g. `?vue=true&style`
- [`resourceFragment`](https://webpack.js.org/api/loaders/#thisresourcefragment): e.g. `#some-fragment`
- inline match resource:

  - Used to redirect the `module.rules` to another, which is able to adjust the loader chain. We will cover this later.
  - Ref: [related PR](https://github.com/webpack/webpack/pull/7462) [Webpack Doc1](https://webpack.js.org/api/loaders/#thisimportmodule) [Webpack Doc2](https://webpack.js.org/api/loaders/#inline-matchresource)

- `virtualResource`:
  - The proposed solution to support asset type changing(A sugar to inline matchResource, which can also affect the asset filename generation)
  - See more: [the background of this property](https://github.com/webpack/webpack/issues/14851)

## Others but also important to note

- Virtual Module: A kind of module that does not locate in the real file system. But you can still import it. To create a virtual module, you need to follow the [spec](https://www.ietf.org/rfc/rfc2397.txt) and it's also worth noting that Node.js and Webpack both support it under the scheme of `data:`. Also known as, `data:` import. [Doc to Node.js](https://nodejs.org/api/esm.html#data-imports)
- [Module types](https://webpack.js.org/concepts/modules/#supported-module-types) with native support: Webpack supports the following module types native: `'javascript/auto'` |` 'javascript/dynamic'` | `'javascript/esm'` | `'json'` | `'webassembly/sync'` | `'webassembly/async'` | `'asset'` | `'asset/source'` | `'asset/resource'` | `'asset/inline'`, for those types you can use it **without a loader**. From webpack version 4.0+, webpack can understand more than `javascript` alone.

# Guide-level explanation

## Loader configuration

The way that webpack controls what kind of module that each loader would apply is based on `module.rules`

```javascript
const MiniExtractCssPlugin = require('mini-extract-css-plugin');

module.exports = {
  module: {
    rules: [
      {
        test: /\.vue$/,
        use: ['vue-loader'],
      },
      {
        test: /\.css$/,
        use: [MiniExtractCssPlugin.loader, 'css-loader'],
      },
    ],
  },
  plugins: [new MiniExtractCssPlugin()],
};
```

Here is a simple option for the configuration of `vue-loader`. `module.rules[number].test` is a part rule to test **whether a rule should be applied**. For `vue-loader` alone, It's kind of confusing how webpack pass the result to the rule of `css`, we will cover this later. But for now, It's good to notice **there is not only a `test` option alone to test if a rule should be applied**. You can find it [here](https://webpack.js.org/configuration/module/#rule) for full conditions supported. Here're some examples of other conditions you can use.

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.vue$/, // of course, test if the file extension match `vue`.
        scheme: 'data', // if the specifier of a request starts with `data:`
        resourceQuery: '?raw', // if the `resourceQuery` matches then the rule will be applied. For this example, it's a great idea to apply a `raw-loader` here.
        type: 'css', // use webpack's native resource handling for css
      },
    ],
  },
};
```

## Examples

### Vue(1 to n)

In a single file component(SFC) of Vue, there are commonly three blocks or more blocks([custom blocks](https://vue-loader.vuejs.org/guide/custom-blocks.html#example)) contained. The basic idea of implementing this loader is to convert it into JavaScript / CSS and let webpack handle the chunk generation(e.g. Style should be generated into a separate `.css` file)

```vue
<template></template>

<style></style>

<script></script>
```

⬇️⬇️⬇️⬇️⬇️⬇️

`Vue-loader` will firstly turn into the `*.vue` file into something like that.

```javascript
import 'script-path-to-vue-sfc';
import 'template-path-to-vue-sfc';
import 'style-path-to-vue-sfc';
```

You may find it weird how webpack handles these imports and build the transformed code. But if I change the code a little bit, you will find the idea.

```javascript
import 'script:path-to-vue-sfc';
import 'template:path-to-vue-sfc';
import 'style:path-to-vue-sfc';
```

and if we tweak the configuration a little bit to this, webpack will know exactly how to work with these import statements.

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.vue$/,
        use: ['vue-loader'],
      },
      {
        scheme: 'script',
        use: ['apply-your-javascript-loader', 'vue-script-extract-loader'],
      },
      {
        scheme: 'template',
        use: ['apply-your-javascript-loader', 'vue-template-extract-loader'],
      },
      {
        scheme: 'style',
        use: ['apply-your-style-loader', 'vue-style-extract-loader'],
      },
    ],
  },
};
```

We added a few loaders to handle the splitting. I know it's still kind of weird here, but please stick with me and we will find a better way out.

- vue-script-extract-loader: extract the `script` block from a SFC file.
- vue-style-extract-loader: extract the `style` block from a SFC file.
- vue-template-extract-loader: extract the `template` block from a SFC file and convert it into JavaScript.

You will find it's really noisy only to transform a `*.vue` file, four loaders were introduced and I believe none of you would like to separate a simple loader into four. It's a real bummer! It will be great to use a single loader `vue-loader` alone. The current vue loader implementation uses resourceQuery to handle this. But how?

#### Loader optimizations I

We know that webpack uses a few conditions to handle whether a rule should be applied. Even with `rule.test` alone, the `this.resourceQuery` is still available to `loaderContext` which developer could access it with `this` in any loader function(Don't worry if you still don't catch this. You will understand this after). Based on that, we change the `rule` to this:

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /.vue$/,
        use: ['vue-loader'],
      },
    ],
  },
};
```

This indicates "If an import specifier is encountered, please pass me to vue-loader"! If you remember the import transformation above, we could adjust the transformation a little bit to this:

**Before**

```javascript
import 'script-path-to-vue-sfc';
import 'template-path-to-vue-sfc';
import 'style-path-to-vue-sfc';
```

**After**

```javascript
import 'path-to-vue-sfc.vue?script=true';
import 'path-to-vue-sfc.vue?template=true';
import 'path-to-vue-sfc.vue?style=true';
```

These requests will match the `test: /.vue$/` above flawlessly and in the loader we can handle like this:

```javascript
// pseudo code only for proofing of the concept
const compiler = require("some-vue-template-compiler")

const loader = function(source) {
  const {
    resourceQuery /* ?script=true or something else */,
    resourcePath /* path-to-vue-sfc.vue */
  } = this

  if (resourceQuery === "?script=true") {
    return compiler.giveMeCodeofScriptBlock(this.resourcePath) // javascript code
  } else if (resourceQuery === "?template=true") {
    return compiler.giveMeCodeofTemplateBlock(this.resourcePath) // javascript code
  } else if (resourceQuery === "?style=true") {
    return compiler.giveMeCodeofStyleBlock(this.resourcePath) // style code
  } else {
    return `
    	import `${this.resourcePath}?script=true`;
    	import `${this.resourcePath}?template=true`;
    	import `${this.resourcePath}?style=true`;
    `
  }
}

module.exports = loader
```

You can see the loader for the example above will be used for four times.

1. Encounter a `*.vue` file, transform the code to a few import statements
2. For each import statement introduced in the first transformation, the loader will be used again as they share the same extension `vue`.

Is this the end? No! Even if you wrote the code like this, it will still fail to load.

1. For CSS: You haven't tell webpack a way to handle the CSS, remember the CSS part is required to go through the `css-loader` and then `mini-css-extract`(if you want to generate CSS for chunk) or `style-loader`(if you want to append it directly to the DOM). After all, you have to make the result of style to pass these loaders.
2. For JS: You haven't transformed the code to any transpilers, It will be failed if your runtime doesn't support the syntax(maybe in TypeScript for example) and webpack internal acorn compiler does not have the ability to help you with that.

**Pass the code to the corresponding loaders**

We tweak the configuration a little bit again.

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /.vue$/,
        use: ['vue-loader'],
      },
      {
        test: /.css$/,
        use: [MiniCssExtractPlugin.loader, 'css-loader'],
      },
      {
        test: /.js$/,
        use: ['babel-loader'],
      },
    ],
  },
};
```

It looks a bit more like the "normal" Webpack configuration. Note that the `rule.test` is based on the file extension, so `vue-loader` did a little bit of hack here.

```javascript
// pseudo code only for proofing of the concept
const compiler = require("some-vue-template-compiler")

const loader = function(source) {
  const {
    resourceQuery /* ?script=true or something else */,
    resourcePath /* path-to-vue-sfc.vue */
  } = this

  if (resourceQuery === "?script=true") {
    const code = compiler.giveMeCodeofScriptBlock(this.resourcePath) // javascript code
    this.resourcePath += ".js"
    return code
  } else if (resourceQuery === "?template=true") {
    const code = compiler.giveMeCodeofTemplateBlock(this.resourcePath) // javascript code
    this.resourcePath += ".js"
    return code
  } else if (resourceQuery === "?style=true") {
    const code = compiler.giveMeCodeofStyleBlock(this.resourcePath) // style code
    this.resourcePath += ".css" // based on the `lang` in each script, the extension will be set accordingly.
    return code
  } else {
    return `
    	import `${this.resourcePath}?script=true`;
    	import `${this.resourcePath}?template=true`;
    	import `${this.resourcePath}?style=true`;
    `
  }
}

module.exports = loader
```

Webpack uses `resourcePath` to match a `module.rules`. So this hack will let webpack treat blocks accordingly as if they are real files with extensions of `js` | `css` |`...` .

Finally! But this is only a proof of concept, for the real implementation. You should definitely check out the [`vue-loader`](https://github.com/vuejs/vue-loader) yourself.

#### Loader Optimization II

Well done! We implemented a simple and rudimentary version of `vue-loader`. However, the real painful part of this implementation is hacking the extension to match the configuration. But since almost every user would have other `js` | `css` files included in the project, so vue team decide to use this kind of strategy to reuse the user configuration.

Except for hacking the extension, webpack then provided a more legit way to handle this kind of **rule matching problem** which is known as **_inline match resource_** (We covered it in the glossary part).

**inline match resource**

Webpack can do almost anything with an import specifier like the loader chaining we covered in the glossary part. _Inline source match_ is another case. By taking the advantage of it, you can force an import statement to go through a `module.rules` by introducing the `!=!` syntax. For example, if we want to force a `css` file to go through a `less` loader, it will be look like this:

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /.less$/,
        use: ['style-loader', 'css-loader', 'less-loader'],
      },
    ],
  },
};
```

```javascript
// This import should be converted with a loader

// treat the file as `less`
import './index.css.less!=!./index.css';
```

The slice before the `!=!` is a way to modify the extension of a single file and force it to match the `module.rules` and this transformation is often done in a loader, or you will make your application code specialized for Webpack only.

After going through the basic example, let's see how we're going to optimize out the hack used in `vue-loader`.

```javascript
// pseudo code only for proofing of the concept
const compiler = require("some-vue-template-compiler")

const loader = function(source) {
  const {
    resourceQuery /* ?script=true or something else */,
    resourcePath /* path-to-vue-sfc.vue */
  } = this

  if (resourceQuery === "?vue=true&script=true") {
    return compiler.giveMeCodeofScriptBlock(this.resourcePath) // javascript code
  } else if (resourceQuery === "?vue=true&template=true") {
    return compiler.giveMeCodeofTemplateBlock(this.resourcePath) // javascript code
  } else if (resourceQuery === "?vue=true&style=true") {
    return compiler.giveMeCodeofStyleBlock(this.resourcePath) // style code
  } else {
    return `
    	import `${this.resourcePath}.js!=!${this.resourcePath}?vue=true&script=true`;
    	import `${this.resourcePath}.js!=!${this.resourcePath}?vue=true&template=true`;
    	import `${this.resourcePath}.css!=!${this.resourcePath}?vue=true&style=true`;
    `
  }
}

module.exports = loader
```

Webpack will internally use the match resource part(before `!=!`) as the data to match loaders. In order to let `vue-loader` match the resource. We have two options:

1. Loose test
2. _Inline loader syntax_

**1. Loose test**

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.vue/, // original: `/\.vue$/`, we removed the `$` to allow resources with `.vue` included to match this rule.
        use: ['vue-loader'],
      },
    ],
  },
};
```

We removed the `$` to allow resources with `.vue` included matching this rule. Personally speaking, this is not a good idea, because a loose match might cause mismatches.

**2. Inline loader syntax**

```javascript
// vue-loader/index.js

module.exports = function() {
 // ... code omitted
	return `
  	import `${this.resourcePath}.js!=!${__filename}!${this.resourcePath}?vue=true&script=true`;
  	import `${this.resourcePath}.js!=!${__filename}!${this.resourcePath}?vue=true&template=true`;
  	import `${this.resourcePath}.css!=!${__filename}!${this.resourcePath}?vue=true&style=true`;
	`
}
```

This technique is to take advantage of the **_inline loader syntax_** to force the loader to go through the vue loader. This tackles down the tangible mismatching ideally and we can still retain the test regex `/\.vue$/` as-is.

#### Final art and conclusion

**Configuration**

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.vue$/,
        use: ['vue-loader'],
      },
      // ... other rules for js, or css, etc.
    ],
  },
};
```

**Loader**

```javascript
// pseudo code only for proofing of the concept
const compiler = require("some-vue-template-compiler")

const loader = function(source) {
  const {
    resourceQuery /* ?script=true or something else */,
    resourcePath /* path-to-vue-sfc.vue */
  } = this

  if (resourceQuery === "?vue=true&script=true") {
    return compiler.giveMeCodeofScriptBlock(resourcePath) // javascript code
  } else if (resourceQuery === "?vue=true&template=true") {
    return compiler.giveMeCodeofTemplateBlock(resourcePath) // javascript code
  } else if (resourceQuery === "?vue=true&style=true") {
    return compiler.giveMeCodeofStyleBlock(resourcePath) // style code
  } else {
    return `
    	import `${this.resourcePath}.js!=!${__filename}!${resourcePath}?vue=true&script=true`;
    	import `${this.resourcePath}.js!=!${__filename}!${resourcePath}?vue=true&template=true`;
    	import `${this.resourcePath}.css!=!${__filename}!${resourcePath}?vue=true&style=true`;
    `
  }
}

module.exports = loader
```

**Conclusion**

Vue-loader is quite complex. The basic needs of the loader are:

1. Separate a `*.vue` file request into a number of parts. For each block, explicitly change the resource matching mechanism (using **_inline match resource_**). The killer _inline match resource_ not only gives us great composability with user-defined loaders, but also the ability to interact with webpack supported native types, and we will cover this part late.
2. When requesting the `vue-loader` again for a block, the code of each block is returned and let webpack handle the changed matched resource(e.g. `./App.vue.css`) with user-defined loaders (Webpack did this internally).

### Use natively supported module types

We know that webpack only supports `JavaScript` in the old time, from the version of `4.0.0`+([changelog](https://github.com/webpack/webpack/releases/tag/v4.0.0))

#### Simplified pre-processor's configuration

> With the experimental support of CSS. A.K.A webpack knows how to handle CSS files natively.

**Before**

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.less$/,
        use: ['style-loader', 'css-loader', 'less-loader'],
        type: 'javascript/auto', // this field is a implicit one, if not defined, it will be set to `"javascript/auto"`
      },
    ],
  },
};
```

**After**

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.less$/,
        use: ['less-loader'],
        type: 'css',
      },
    ],
  },
  experiments: {
    css: true,
  },
};
```

With `experiments.css` on, webpack can experimentally understand the parsing and generating of `css` files which gets rid of `css-loader` and `style-loader`. For the full list of natively supported `Rule.type`, you can find it [here](https://webpack.js.org/configuration/module/#ruletype).

#### Asset modules

> From _webpack 4.0.0+_, assets are supported natively

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.(png|jpg)/,
        type: 'asset',
      },
    ],
  },
};
```

`Rule.type === "asset"` indicates the asset will be automatically tested whether it's going to be inlined or emitted as a file on the real file system. The possible options are: `'asset'` | `'asset/source'` | `'asset/resource'` | `'asset/inline'`

### Svgr

Webpack loader will read the source to a UTF-8 string by default. For SVG files, this would fit the webpack load defaults.

```javascript
// Proof of concept of svgr-loader
module.exports = function (source) {
  if (this.resourceQuery === '?svgr=true') {
    // the real transform part
    let { code } = svgrTransformer.transform(source);
    return code;
  }
  return `require("${this.resourcePath}.jsx!=!${__filename}!${this.resourcePath}?svgr=true")`; // the request part
};
```

Again here we use double-pass to firstly convert each request to the request part with _inline match resource_, and do the real request with query `?svgr=true`, and let _inline match resource_ handle the `jsx` conversion. Before that, we have to call a third-party `jsx` transformer, could be _ESBuild_ for example, for which we cannot reuse other `module.rules` set by the user-side. _Inline match resource_ saved our pain again!

### Scheme imports

> Supported in _Webpack version 5.38.0_, doc: [Rule.scheme](https://webpack.js.org/configuration/module/#rulescheme)

```javascript
// JavaScript
import x from 'data:text/javascript,export default 42';
console.log('x:', x);
```

```css
/* CSS */
@import ('data:text/css, body { background: #fff; }');
```

Webpack handles `data:` imports for JavaScript internally.

### Asset transform and rename

> [**Asset**](https://webpack.js.org/guides/asset-management/): This is a general term for the images, fonts, media, and any other kind of files that are typically used in websites and other applications. These typically end up as individual files within the [output](https://webpack.js.org/glossary/#o) but can also be inlined via things like the [style-loader](https://webpack.js.org/loaders/style-loader) or [url-loader](https://webpack.js.org/loaders/url-loader).
>
> _Originally posted at Webpack [Glossary](https://webpack.js.org/glossary/#a)_

#### Default resource reading override

Asset could be formatted in both text(`*.svg`) or binary (`*.png` / `*.jpg`). For loaders, webpack provides you an option [`raw`](https://webpack.js.org/api/loaders/#raw-loader) to override the default and built-in resource reading strategy from UTF-8 `string` to `Buffer`:

```javascript
module.exports = function (source /* Buffer */) {
  // loader implementation
};

module.exports.raw = true;
```

#### Transform and rename

Image there is a need to transform an asset formatted with `png` to `jpg`. There is two abilities that webpack needs to support:

1. Handle the asset with `raw` content, or a `Buffer`. We can simply override the default resource reading behavior by exporting `raw`(covered before).
2. Change the filename, and reuse the loader for both `png` and `jpg`

##### Configuration

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.png/,
        use: ["png-to-jpg-loader"] // some png to jpg loader, we will implement this
      },
      {
        test: /\.jpg/,
        use: ["jpg-optimizer"] // some jpg optimizer, we will not covert this,
        type: "asset/resource"
      }
    ]
  }
}
```

1. Rule1: For files with extension `png`, we want to use a `png` to `jpg` loader, which will be covered in this article.
2. Rule2:
   1. For files with extension `jpg`, we want to use a third-party `jpg-optimizer`, which will not be covered in this article.
   2. `type: "asset/resource"`: As soon as all the loaders have gone through, we want webpack to emit the file as an external resource on the file system regardless of the file size(`type: "asset"` will automatically detect the size of an asset to determine whether an asset will be inline-included for dynamically imported from file system).
3. For those `jpg` files converted from `png`, we want them to apply with the `jpg-optimizer` too(i.e. reuse the loaders defined in `module.rules`)

##### Loader

```javascript
module.exports = function (source) {
  if (this.resourceQuery === '?pngToJPG=true') {
    return pngToJpg.transform(source);
  }

  return `require("${this.resourcePath}.jpg!=!${__filename}${this.resourcePath}?pngToJPG=true")`;
};

module.exports.raw = true;
```

We use double-pass again, firstly we convert the extension to `.jpg` which will apply the matched rules(in this case `test: /\.jpg/`), after the transformation of `png-to-jpg-loader`. Generated asset module filename will be based on the _inline match resource_, which is `xxxx.jpg` in this case.

### AST reuse

Webpack provides a way to pass metadata(the forth parameter) among the chaining loaders [doc](https://webpack.js.org/api/loaders/#thiscallback). The most commonly used value is `webpackAST` which accepts an `ESTree` compatible(webpack internally uses `acorn`) AST, which hugely improves the performance since webpack instead of parsing the returned code to AST again, **will directly use the AST(`webpackAST`) returned from a loader**(But **the work of a complete walking of an AST can not be omitted** as it's necessary for webpack for do some analysis for its dependencies and will be only done once, so it is not a big overhead.)

```javascript
module.exports = function (source) {
  let ast = AcornParser.parse(source, {
    // options
  });

  this.callback(null, '', null, {
    webpackAST: ast,
  });
};
```

Good to note that only `ESTree` is compatible, so you cannot pass a CSS AST, or webpack will complain with `"webpackAst is unexpected for the CssParser"`. It will be ok if you don't get this, let's move to the reference-level explanation for analysis in-depth.

## Reference-level explanation

This is the reference-level explanation part of webpack's internal loader implementation.

### Loader composability

> If you don't quite get this concept, you may refer to the Glossary and _Example_ part of the Guide-level explanation first and pick up this as soon as you finished.

The high-level idea of previously talked _inline match resource_ is to let **loader developers** to customize the behavior of matching to match the pre-defined `module.rules`. It's an API to write composable loaders. But what does composition mean? For those users who are familiar with React hooks and Vue composable APIs, you may get this faster. Actually, webpack provides a lot of ways to help loader developers and users do the composition.

#### User-defined loader flows

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.js$/,
        use: ['babel-loader'],
        type: 'javascript/auto',
      },
      {
        test: /\.svg$/,
        use: ['svgr-loader', 'svgo-loader'],
      },
    ],
  },
};
```

Webpack users can take the advantage of `module.rules[number].use` with a loader list for each request that matches the corresponding conditions. Note that I use the wording of `request,` not the `file` , which can include a request to `data:text/javascript` not the files on the real file system only. (In Parcel bundler, it's called [_pipelines_](https://parceljs.org/features/plugins/#pipelines), but this will not be covered in this article.)

Apparently, user-declared loader flow is not able to cover up every case that a loader wants. You can see from the previous examples, `vue-loader` wants to split a file into many blocks, and remain the reference to it. `svgr-loader` wants to do the transformation first and let other loaders deal with the `jsx`. `svg-loader` wants to use the internal ability of `Asset Module` to let Webpack decide whether an asset is inlined or emitted to the real file system. and there are more to come... Based on the complexity of the loader, Webpack also provides a syntax to allow loader implementors to do the composition by themselves.

#### The syntax for loader composition

##### Inline loader syntax (Chaining loaders)

> Supported from _webpack v1_ [chaining-loaders](https://webpack.js.org/migrate/3/#chaining-loaders)
>
> It's possible to specify loaders in an `import` statement, or any [equivalent "importing" method](https://webpack.js.org/api/module-methods). Separate loaders from the resource with `!`. Each part is resolved relative to the current directory. [doc](https://webpack.js.org/concepts/loaders/#inline)

```javascript
import Styles from '!style-loader!css-loader?modules!./styles.css';
```

The _inline loader syntax_ executes each loader for each request from right to left. Webpack handles the interaction with user-defined loaders carefully. So by default, the user-defined normal loader will be executed prior to the inline loaders, you can disable this behavior by prefixing `!` , (full reference could be found here [doc](https://webpack.js.org/concepts/loaders/#inline)).

The custom specifier is parsed before the `module.rules` as the _inline loader syntax_ interferes the user-defined loaders(See the [source code](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/NormalModuleFactory.js#L390-L403)). Then, webpack will get the `module.rules` combined with the required conditions to calculate the matching rule set (See the [source code](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/NormalModuleFactory.js#L493-L510)).

At the moment, you cannot change the matching behavior with the syntax, loaders are always matched with the provided _resourcePath_, etc, which leads to a bunch of hack code in the implementations of loaders (see this [code snippet](https://github.com/vuejs/vue-loader/blob/e9314347d75a1b0e54f971272d23a669fc3e6965/src/select.ts#L31) in `vue-loader`). The possibilities for changing the matching behavior leaves to the later-coming _inline match resource_.

Nevertheless, the architecture of Loader at this moment is sound and solid. Another good example is the implementation-nonrelative filter(i.e. the filtering logic of _Loader_ is not declared in the loader itself), which is the fundamental root of loader composition, or the implementor will do a lot of hacks. (It's way too dirty to talk about here, but you can take the rollup [svgr](https://github.com/gregberge/svgr/blob/1dbc3e2c2027253b3b81b92fd4eb09a4aa8ae25e/packages/rollup/src/index.ts#L52) plugin as a reference)

In conclusion, _inline loader syntax_ gives us a chance to control the loader flow with user-defined rules.

##### Inline match resource

To extend the matching ability, _inline match resource_ enables loader implementors to reuse some of the user-defined configurations with more flexibilities.

On top of the previous example, webpack also provides a way to make use of the natively-supported _module types_.

```javascript
// For module type `css` to work, you need to enable `experiments.css`
import './style.less.webpack[css]!=path-to-less-loader!./style.less';
```

```javascript
// webpack.config.js
module.exports = {
  experiments: {
    css: true,
  },
};
```

Given the configuration above, the overview of the complete flow will be like this:

1. Webpack: Parse the specifier of the import and create the loader for the current request
2. Webpack: Merge the result from the second step with a user-defined `module.rules` in `webpack.config`, in this case is `[]`
3. Webpack: load `style.less` as UTF-8 string
4. Less-loader: Accept the UTF-8 string as the first parameter of the loader function and transform it to the content of `css`.
5. Webpack: Call the registered native `CSS` parser, and later at the code generation step the registered native `CSS` generator generates the result.

For _asset modules_, you can also use this:

```javascript
import './logo.png.jpg.webpack[asset/resource]!=path-to-loaders!./logo.png';
```

The first part, also known as `matchResource` will be used as a part of the `filename` of the final code generation. (See the [source code](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/asset/AssetGenerator.js#L293-L348))

### Performance optimizations

Before moving on to the detailed implementations, here's some glossary to support your understanding the architecture as a whole.

#### Glossary

- `NormalModuleFactory`: A factory used to create a `NormalModule`, which basically exposes a `create` method.
- `NormalModule`: A module in Webpack most of the time is a `NormalModule`, but with different implementations of `parser`/ `generator` / `Module Type`, the module could be almost any kind, and also exposes a `build` method. For example, a `NormalModule` with JavaScript parser, JavaScript generator, and `type ===javascript/auto` will be regarded as a module with JavaScript-related functionalities. Also, good to note that a module may not exist on the real file system, taking `data:` for example.

#### The module creation workflow

> This will only introduce a slice of webpack's internal implementation from **the Loader's perspective**, for more you should directly refer to the source code.

When an import statement is detected, webpack will initialize a module creation. Based on the type of _Dependency_ (an abstraction of webpack, it's not important here), webpack can find the linked _ModuleFactory_(The abstraction class), in most cases, the derived factory is `NormalModuleFactory`, which exposes a `create` method.

##### Prepare data needed for module creation

The `NormalModuleFactory#create` is used to provide enough information to create a real `NormalModule`, and create the `NormalModule`. In the `create` method, webpack basically does these things(some non-loader related stuff will be omitted):

- Resolve loaders from request: resolve the request, parse inline loader syntax: This contains _inline match resource_, _inline loader syntax_.
- Do the analysis on the parsed loader syntax, to decide whether a user-defined `normal/post/pre` loader is going to be included. [doc](https://webpack.js.org/concepts/loaders/#inline)
- Resolve Resource: resolve resource to the absolute path, fragments, queries, etc(These stuff are also provided in `LoaderContext`). For the full source code you may refer to [this](https://github.com/webpack/webpack/blob/main/lib/NormalModuleFactory.js#L653-L678)
- Use the resolved resource data to match `module.rules` defined in the configuration, and get the matched rules. This is also a part of the module creation data.
- Do some special logic with _inline match resource_, since match resource ends like `.webpack[css]` would change `Rule.type`. Also store the match resource data, since it might affect the filename generation for _asset modules_.

##### Create a module based on the prepared data

After the data needed for module creation is prepared, `NormalModuleFactory` will `new NormalModule` with the data provided. It contains basically every that a `NormalModule` needs (see the [source code](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/NormalModule.js#L271-L287)). Most importantly, the `loaders`. It contains every loader parsed and ordered from the `create` step.

#### The module build step

The module build step is kind of clear. Webpack will invoke the `build` method for each `NormalModule` instance, which invokes `loader-runner`(see the [source code](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/NormalModule.js#L819)) to go through every loader that was analyzed from the create step. It's clear to **know that the composition of loaders is happening on the same module**.

#### A peek of the support of _Module Types_

As far as this article goes, It might be getting a little bit tedious. But have you ever wondered how webpack supports these _module types_ natively? I think It's still worth telling you about it to get a more complete understanding of the AST optimizations. For the support of JavaScript, webpack's JavaScript plugin will register different types of parser and generators for each _module types_, which will be used as the `parser` / `generator` to a `NormalModule` (see the [source code](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/javascript/JavascriptModulesPlugin.js#L202-L231)).

#### Reusing AST in Webpack

Based on the parser and generator we introduced before, webpack did a little hack around the fourth parameter of `this.callback` (from _loaderContext_), with `webpackAST`, after each loader call, the `webpackAST` will be stored in the context of loader, and passed again to the next loader. Finally, the AST will be passed to the `parser`(It could be any type, based on the _module type_, but webpack makes it a JavaScript only for AST) (see the [source code](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/NormalModule.js#L1087)).

Here's an issue about trying to use SWC 's AST to get rid of the time sensitive code parsing from Acorn Parser, but they are facing some AST compatibility issues and performance issues about the overhead of interop with native code(Rust).

## References

- loader plugin api design (Analysis) [#315](https://github.com/speedy-js/rspack/discussions/315)

- RFC-011 Supports `data:text/javascript` protocol [#457](https://github.com/speedy-js/rspack/discussions/457)

- Webpack: `matchResource` with natively-supported module types [doc](https://webpack.js.org/api/loaders/#thisimportmodule)

- Webpack: Loader context [doc](https://webpack.js.org/api/loaders/#the-loader-context)

- Webpack: Module rules [doc](https://webpack.js.org/configuration/module/#rule)

- SWC-loader for performance optimizations [issue](https://github.com/webpack/webpack/issues/13425#issuecomment-1013560170)
