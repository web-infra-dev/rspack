/**
 * Some types are modified from https://github.com/swc-project/swc/blob/16a38851/packages/types/index.ts#L647
 * license at https://github.com/swc-project/swc/blob/main/LICENSE
 */
import type { ReactOptions } from "./react";
import type { RelayOptions } from "./relay";
import type { EmotionOptions } from "./emotion";
import type { PluginImportOptions } from "./pluginImport";

export type StyledComponentsOptions = {
	displayName?: boolean;
	ssr?: boolean;
	fileName?: boolean;
	meaninglessFileNames?: string[];
	namespace?: string;
	topLevelImportPaths?: string[];
	transpileTemplateLiterals?: boolean;
	minify?: boolean;
	pure?: boolean;
	cssProps?: boolean;
};

export type JscTarget =
	| "es3"
	| "es5"
	| "es2015"
	| "es2016"
	| "es2017"
	| "es2018"
	| "es2019"
	| "es2020"
	| "es2021"
	| "es2022"
	| "esnext";

export type SwcLoaderParserConfig =
	| SwcLoaderTsParserConfig
	| SwcLoaderEsParserConfig;

export interface SwcLoaderTsParserConfig {
	syntax: "typescript";
	/**
	 * Defaults to `false`.
	 */
	tsx?: boolean;
	/**
	 * Defaults to `false`.
	 */
	decorators?: boolean;
	/**
	 * Defaults to `false`
	 */
	dynamicImport?: boolean;
}

export interface SwcLoaderEsParserConfig {
	syntax: "ecmascript";
	/**
	 * Defaults to false.
	 */
	jsx?: boolean;
	/**
	 * Defaults to `false`
	 */
	functionBind?: boolean;
	/**
	 * Defaults to `false`
	 */
	decorators?: boolean;
	/**
	 * Defaults to `false`
	 */
	decoratorsBeforeExport?: boolean;
	/**
	 * Defaults to `false`
	 */
	exportDefaultFrom?: boolean;
	/**
	 * Defaults to `false`
	 */
	importAssertions?: boolean;
}

/**
 *  - `import { DEBUG } from '@ember/env-flags';`
 *  - `import { FEATURE_A, FEATURE_B } from '@ember/features';`
 *
 * See: https://github.com/swc-project/swc/issues/18#issuecomment-466272558
 */
export type ConstModulesConfig = {
	globals?: {
		[module: string]: {
			[name: string]: string;
		};
	};
};

/**
 * Options for inline-global pass.
 */
export interface GlobalPassOption {
	/**
	 * Global variables that should be inlined with passed value.
	 *
	 * e.g. `{ __DEBUG__: true }`
	 */
	vars?: Record<string, string>;
	/**
	 * Names of environment variables that should be inlined with the value of corresponding env during build.
	 *
	 * Defaults to `["NODE_ENV", "SWC_ENV"]`
	 */
	envs?: string[] | Record<string, string>;
	/**
	 * Replaces typeof calls for passed variables with corresponding value
	 *
	 * e.g. `{ window: 'object' }`
	 */
	typeofs?: Record<string, string>;
}

/**
 * https://swc.rs/docs/configuring-swc.html#jsctransformoptimizerjsonify
 */
export type OptimizerConfig = {
	/**
	 * https://swc.rs/docs/configuration/compilation#jsctransformoptimizersimplify
	 */
	simplify?: boolean;
	/**
	 * https://swc.rs/docs/configuring-swc.html#jsctransformoptimizerglobals
	 */
	globals?: GlobalPassOption;
	/**
	 * https://swc.rs/docs/configuring-swc.html#jsctransformoptimizerjsonify
	 */
	jsonify?: { minCost: number };
};

export interface SwcLoaderTransformConfig {
	/**
	 * Effective only if `syntax` supports ƒ.
	 */
	react?: ReactOptions;
	constModules?: ConstModulesConfig;
	/**
	 * Defaults to null, which skips optimizer pass.
	 */
	optimizer?: OptimizerConfig;
	/**
	 * https://swc.rs/docs/configuring-swc.html#jsctransformlegacydecorator
	 */
	legacyDecorator?: boolean;
	/**
	 * https://swc.rs/docs/configuring-swc.html#jsctransformdecoratormetadata
	 */
	decoratorMetadata?: boolean;
	/**
	 * https://swc.rs/docs/configuration/compilation#jsctransformdecoratorversion
	 */
	decoratorVersion?: "2021-12" | "2022-03";
	treatConstEnumAsEnum?: boolean;
	useDefineForClassFields?: boolean;
}

export interface SwcLoaderEnvConfig {
	mode?: "usage" | "entry";
	debug?: boolean;
	dynamicImport?: boolean;
	loose?: boolean;
	/**
	 * Transpiles the broken syntax to the closest non-broken modern syntax
	 * Defaults to false.
	 */
	bugfixes?: boolean;
	/**
	 * Skipped es features.
	 * e.g.)
	 *   - `core-js/modules/foo`
	 */
	skip?: string[];
	include?: string[];
	exclude?: string[];
	/**
	 * The version of the used core js.
	 */
	coreJs?: string;
	targets?: any;
	path?: string;
	shippedProposals?: boolean;
	/**
	 * Enable all transforms
	 */
	forceAllTransforms?: boolean;
}

export interface SwcLoaderJscConfig {
	loose?: boolean;
	/**
	 * Defaults to EsParserConfig
	 */
	parser?: SwcLoaderParserConfig;
	transform?: SwcLoaderTransformConfig;
	/**
	 * Use `@swc/helpers` instead of inline helpers.
	 */
	externalHelpers?: boolean;
	/**
	 * Defaults to `es3` (which enabled **all** pass).
	 */
	target?: JscTarget;
	/**
	 * Keep class names.
	 */
	keepClassNames?: boolean;
	/**
	 * This is experimental, and can be removed without a major version bump.
	 */
	experimental?: {
		optimizeHygiene?: boolean;
		/**
		 * Preserve `with` in imports and exports.
		 */
		keepImportAttributes?: boolean;
		/**
		 * Use `assert` instead of `with` for imports and exports.
		 * This option only works when `keepImportAttributes` is `true`.
		 */
		emitAssertForImportAttributes?: boolean;
		/**
		 * Specify the location where SWC stores its intermediate cache files.
		 * Currently only transform plugin uses this. If not specified, SWC will
		 * create `.swc` directories.
		 */
		cacheRoot?: string;
		/**
		 * List of custom transform plugins written in WebAssembly.
		 * First parameter of tuple indicates the name of the plugin - it can be either
		 * a name of the npm package can be resolved, or absolute path to .wasm binary.
		 *
		 * Second parameter of tuple is JSON based configuration for the plugin.
		 */
		plugins?: Array<[string, Record<string, any>]>;
		/**
		 * Disable builtin transforms. If enabled, only Wasm plugins are used.
		 */
		disableBuiltinTransformsForInternalTesting?: boolean;
	};
	baseUrl?: string;
	paths?: {
		[from: string]: [string];
	};
	preserveAllComments?: boolean;
}

export type SwcLoaderModuleConfig =
	| Es6Config
	| CommonJsConfig
	| UmdConfig
	| AmdConfig
	| NodeNextConfig
	| SystemjsConfig;

export interface BaseModuleConfig {
	/**
	 * By default, when using exports with babel a non-enumerable `__esModule`
	 * property is exported. In some cases this property is used to determine
	 * if the import is the default export or if it contains the default export.
	 *
	 * In order to prevent the __esModule property from being exported, you
	 *  can set the strict option to true.
	 *
	 * Defaults to `false`.
	 */
	strict?: boolean;
	/**
	 * Emits 'use strict' directive.
	 *
	 * Defaults to `true`.
	 */
	strictMode?: boolean;
	/**
	 * Changes Babel's compiled import statements to be lazily evaluated when their imported bindings are used for the first time.
	 *
	 * This can improve initial load time of your module because evaluating dependencies up
	 *  front is sometimes entirely un-necessary. This is especially the case when implementing
	 *  a library module.
	 *
	 *
	 * The value of `lazy` has a few possible effects:
	 *
	 *  - `false` - No lazy initialization of any imported module.
	 *  - `true` - Do not lazy-initialize local `./foo` imports, but lazy-init `foo` dependencies.
	 *
	 * Local paths are much more likely to have circular dependencies, which may break if loaded lazily,
	 * so they are not lazy by default, whereas dependencies between independent modules are rarely cyclical.
	 *
	 *  - `Array<string>` - Lazy-initialize all imports with source matching one of the given strings.
	 *
	 * -----
	 *
	 * The two cases where imports can never be lazy are:
	 *
	 *  - `import "foo";`
	 *
	 * Side-effect imports are automatically non-lazy since their very existence means
	 *  that there is no binding to later kick off initialization.
	 *
	 *  - `export * from "foo"`
	 *
	 * Re-exporting all names requires up-front execution because otherwise there is no
	 * way to know what names need to be exported.
	 *
	 * Defaults to `false`.
	 */
	lazy?: boolean | string[];
	/**
	 * @deprecated  Use the `importInterop` option instead.
	 *
	 * By default, when using exports with swc a non-enumerable __esModule property is exported.
	 * This property is then used to determine if the import is the default export or if
	 *  it contains the default export.
	 *
	 * In cases where the auto-unwrapping of default is not needed, you can set the noInterop option
	 *  to true to avoid the usage of the interopRequireDefault helper (shown in inline form above).
	 *
	 * Defaults to `false`.
	 */
	noInterop?: boolean;
	/**
	 * Defaults to `swc`.
	 *
	 * CommonJS modules and ECMAScript modules are not fully compatible.
	 * However, compilers, bundlers and JavaScript runtimes developed different strategies
	 * to make them work together as well as possible.
	 *
	 * - `swc` (alias: `babel`)
	 *
	 * When using exports with `swc` a non-enumerable `__esModule` property is exported
	 * This property is then used to determine if the import is the default export
	 * or if it contains the default export.
	 *
	 * ```javascript
	 * import foo from "foo";
	 * import { bar } from "bar";
	 * foo;
	 * bar;
	 *
	 * // Is compiled to ...
	 *
	 * "use strict";
	 *
	 * function _interop_require_default(obj) {
	 *   return obj && obj.__esModule ? obj : { default: obj };
	 * }
	 *
	 * var _foo = _interop_require_default(require("foo"));
	 * var _bar = require("bar");
	 *
	 * _foo.default;
	 * _bar.bar;
	 * ```
	 *
	 * When this import interop is used, if both the imported and the importer module are compiled
	 * with swc they behave as if none of them was compiled.
	 *
	 * This is the default behavior.
	 *
	 * - `node`
	 *
	 * When importing CommonJS files (either directly written in CommonJS, or generated with a compiler)
	 * Node.js always binds the `default` export to the value of `module.exports`.
	 *
	 * ```javascript
	 * import foo from "foo";
	 * import { bar } from "bar";
	 * foo;
	 * bar;
	 *
	 * // Is compiled to ...
	 *
	 * "use strict";
	 *
	 * var _foo = require("foo");
	 * var _bar = require("bar");
	 *
	 * _foo;
	 * _bar.bar;
	 * ```
	 * This is not exactly the same as what Node.js does since swc allows accessing any property of `module.exports`
	 * as a named export, while Node.js only allows importing statically analyzable properties of `module.exports`.
	 * However, any import working in Node.js will also work when compiled with swc using `importInterop: "node"`.
	 *
	 * - `none`
	 *
	 * If you know that the imported file has been transformed with a compiler that stores the `default` export on
	 * `exports.default` (such as swc or Babel), you can safely omit the `_interop_require_default` helper.
	 *
	 * ```javascript
	 * import foo from "foo";
	 * import { bar } from "bar";
	 * foo;
	 * bar;
	 *
	 * // Is compiled to ...
	 *
	 * "use strict";
	 *
	 * var _foo = require("foo");
	 * var _bar = require("bar");
	 *
	 * _foo.default;
	 * _bar.bar;
	 * ```
	 */
	importInterop?: "swc" | "babel" | "node" | "none";
	/**
	 * Emits `cjs-module-lexer` annotation
	 * `cjs-module-lexer` is used in Node.js core for detecting the named exports available when importing a CJS module into ESM.
	 * swc will emit `cjs-module-lexer` detectable annotation with this option enabled.
	 *
	 * Defaults to `true` if import_interop is Node, else `false`
	 */
	exportInteropAnnotation?: boolean;
	/**
	 * If set to true, dynamic imports will be preserved.
	 */
	ignoreDynamic?: boolean;
	allowTopLevelThis?: boolean;
	preserveImportMeta?: boolean;
}

export interface Es6Config extends BaseModuleConfig {
	type: "es6";
}

export interface NodeNextConfig extends BaseModuleConfig {
	type: "nodenext";
}

export interface CommonJsConfig extends BaseModuleConfig {
	type: "commonjs";
}

export interface UmdConfig extends BaseModuleConfig {
	type: "umd";
	globals?: { [key: string]: string };
}

export interface AmdConfig extends BaseModuleConfig {
	type: "amd";
	moduleId?: string;
}

export interface SystemjsConfig {
	type: "systemjs";
	allowTopLevelThis?: boolean;
}

export type SwcLoaderOptions = {
	/**
	 * Note: The type is string because it follows rust's regex syntax.
	 */
	test?: string | string[];
	/**
	 * Note: The type is string because it follows rust's regex syntax.
	 */
	exclude?: string | string[];
	env?: SwcLoaderEnvConfig;
	jsc?: SwcLoaderJscConfig;
	module?: SwcLoaderModuleConfig;
	minify?: boolean;
	/**
	 * - true to generate a sourcemap for the code and include it in the result object.
	 * - "inline" to generate a sourcemap and append it as a data URL to the end of the code, but not include it in the result object.
	 *
	 * `swc-cli` overloads some of these to also affect how maps are written to disk:
	 *
	 * - true will write the map to a .map file on disk
	 * - "inline" will write the file directly, so it will have a data: containing the map
	 * - Note: These options are bit weird, so it may make the most sense to just use true
	 *  and handle the rest in your own code, depending on your use case.
	 */
	sourceMaps?: boolean;
	inlineSourcesContent?: boolean;
	/**
	 * Experimental features provided by Rspack.
	 * @experimental
	 */
	rspackExperiments?: {
		relay?: RelayOptions;
		emotion?: EmotionOptions;
		import?: PluginImportOptions[];
		styledComponents?: StyledComponentsOptions;
	};
};
