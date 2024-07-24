/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/v5.88.2/lib/RuntimeGlobals.js
 *
 * MIT Licensed
 * Author Tobias Koppers \@sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { JsRuntimeGlobals } from "@rspack/binding";

const REVERSE_RUNTIME_GLOBALS = new Map<string, string>();

export function __from_binding_runtime_globals(
	runtimeRequirements: JsRuntimeGlobals
): Set<string> {
	const res = new Set<string>();

	for (const flag of runtimeRequirements.value) {
		if (flag in RuntimeGlobals) {
			res.add(RuntimeGlobals[flag as keyof typeof RuntimeGlobals]);
		} else {
			res.add(flag);
		}
	}

	return res;
}

export function __to_binding_runtime_globals(
	runtimeRequirements: Set<string>
): JsRuntimeGlobals {
	const res: JsRuntimeGlobals = {
		value: []
	};

	for (const flag of Array.from(runtimeRequirements)) {
		const item = REVERSE_RUNTIME_GLOBALS.get(flag);
		if (typeof item === "string") {
			res.value.push(item);
		} else {
			res.value.push(flag);
		}
	}

	return res;
}

export const RuntimeGlobals = {
	/**
	 * the internal require function
	 */
	require: "__webpack_require__",

	/**
	 * access to properties of the internal require function/object
	 */
	requireScope: "__webpack_require__.*",

	/**
	 * the internal exports object
	 */
	exports: "__webpack_exports__",

	/**
	 * top-level this need to be the exports object
	 */
	thisAsExports: "top-level-this-exports",

	/**
	 * runtime need to return the exports of the last entry module
	 */
	returnExportsFromRuntime: "return-exports-from-runtime",

	/**
	 * the internal module object
	 */
	module: "module",

	/**
	 * the internal module object
	 */
	moduleId: "module.id",

	/**
	 * the internal module object
	 */
	moduleLoaded: "module.loaded",

	/**
	 * the bundle public path
	 */
	publicPath: "__webpack_require__.p",

	/**
	 * the module id of the entry point
	 */
	entryModuleId: "__webpack_require__.s",

	/**
	 * the module cache
	 */
	moduleCache: "__webpack_require__.c",

	/**
	 * the module functions
	 */
	moduleFactories: "__webpack_require__.m",

	/**
	 * the module functions, with only write access
	 */
	moduleFactoriesAddOnly: "__webpack_require__.m (add only)",

	/**
	 * the chunk ensure function
	 */
	ensureChunk: "__webpack_require__.e",

	/**
	 * an object with handlers to ensure a chunk
	 */
	ensureChunkHandlers: "__webpack_require__.f",

	/**
	 * a runtime requirement if ensureChunkHandlers should include loading of chunk needed for entries
	 */
	ensureChunkIncludeEntries: "__webpack_require__.f (include entries)",

	/**
	 * the chunk prefetch function
	 */
	prefetchChunk: "__webpack_require__.E",

	/**
	 * an object with handlers to prefetch a chunk
	 */
	prefetchChunkHandlers: "__webpack_require__.F",

	/**
	 * the chunk preload function
	 */
	preloadChunk: "__webpack_require__.G",

	/**
	 * an object with handlers to preload a chunk
	 */
	preloadChunkHandlers: "__webpack_require__.H",

	/**
	 * the exported property define getters function
	 */
	definePropertyGetters: "__webpack_require__.d",

	/**
	 * define compatibility on export
	 */
	makeNamespaceObject: "__webpack_require__.r",

	/**
	 * create a fake namespace object
	 */
	createFakeNamespaceObject: "__webpack_require__.t",

	/**
	 * compatibility get default export
	 */
	compatGetDefaultExport: "__webpack_require__.n",

	/**
	 * harmony module decorator
	 */
	harmonyModuleDecorator: "__webpack_require__.hmd",

	/**
	 * node.js module decorator
	 */
	nodeModuleDecorator: "__webpack_require__.nmd",

	/**
	 * the webpack hash
	 */
	getFullHash: "__webpack_require__.h",

	/**
	 * an object containing all installed WebAssembly.Instance export objects keyed by module id
	 */
	wasmInstances: "__webpack_require__.w",

	/**
	 * instantiate a wasm instance from module exports object, id, hash and importsObject
	 */
	instantiateWasm: "__webpack_require__.v",

	/**
	 * the uncaught error handler for the webpack runtime
	 */
	uncaughtErrorHandler: "__webpack_require__.oe",

	/**
	 * the script nonce
	 */
	scriptNonce: "__webpack_require__.nc",

	/**
	 * function to load a script tag.
	 * Arguments: (url: string, done: (event) =\> void), key?: string | number, chunkId?: string | number) =\> void
	 * done function is called when loading has finished or timeout occurred.
	 * It will attach to existing script tags with data-webpack == uniqueName + ":" + key or src == url.
	 */
	loadScript: "__webpack_require__.l",

	/**
	 * function to promote a string to a TrustedScript using webpack's Trusted
	 * Types policy
	 * Arguments: (script: string) =\> TrustedScript
	 */
	createScript: "__webpack_require__.ts",

	/**
	 * function to promote a string to a TrustedScriptURL using webpack's Trusted
	 * Types policy
	 * Arguments: (url: string) =\> TrustedScriptURL
	 */
	createScriptUrl: "__webpack_require__.tu",

	/**
	 * function to return webpack's Trusted Types policy
	 * Arguments: () =\> TrustedTypePolicy
	 */
	getTrustedTypesPolicy: "__webpack_require__.tt",

	/**
	 * a flag when a chunk has a fetch priority
	 */
	hasFetchPriority: "has fetch priority",

	/**
	 * the chunk name of the chunk with the runtime
	 */
	chunkName: "__webpack_require__.cn",

	/**
	 * the runtime id of the current runtime
	 */
	runtimeId: "__webpack_require__.j",

	/**
	 * the filename of the script part of the chunk
	 */
	getChunkScriptFilename: "__webpack_require__.u",

	/**
	 * the filename of the css part of the chunk
	 */
	getChunkCssFilename: "__webpack_require__.k",

	/**
	 * a flag when a module/chunk/tree has css modules
	 */
	hasCssModules: "has css modules",

	/**
	 * the filename of the script part of the hot update chunk
	 */
	getChunkUpdateScriptFilename: "__webpack_require__.hu",

	/**
	 * the filename of the css part of the hot update chunk
	 */
	getChunkUpdateCssFilename: "__webpack_require__.hk",

	/**
	 * startup signal from runtime
	 * This will be called when the runtime chunk has been loaded.
	 */
	startup: "__webpack_require__.x",

	/**
	 * @deprecated
	 * creating a default startup function with the entry modules
	 */
	startupNoDefault: "__webpack_require__.x (no default handler)",

	/**
	 * startup signal from runtime but only used to add logic after the startup
	 */
	startupOnlyAfter: "__webpack_require__.x (only after)",

	/**
	 * startup signal from runtime but only used to add sync logic before the startup
	 */
	startupOnlyBefore: "__webpack_require__.x (only before)",

	/**
	 * global callback functions for installing chunks
	 */
	chunkCallback: "webpackChunk",

	/**
	 * method to startup an entrypoint with needed chunks.
	 * Signature: (moduleId: Id, chunkIds: Id[]) =\> any.
	 * Returns the exports of the module or a Promise
	 */
	startupEntrypoint: "__webpack_require__.X",

	/**
	 * register deferred code, which will run when certain
	 * chunks are loaded.
	 * Signature: (chunkIds: Id[], fn: () =\> any, priority: int \>= 0 = 0) =\> any
	 * Returned value will be returned directly when all chunks are already loaded
	 * When (priority & 1) it will wait for all other handlers with lower priority to
	 * be executed before itself is executed
	 */
	onChunksLoaded: "__webpack_require__.O",

	/**
	 * method to install a chunk that was loaded somehow
	 * Signature: (\{ id, ids, modules, runtime \}) =\> void
	 */
	externalInstallChunk: "__webpack_require__.C",

	/**
	 * interceptor for module executions
	 */
	interceptModuleExecution: "__webpack_require__.i",

	/**
	 * the global object
	 */
	global: "__webpack_require__.g",

	/**
	 * an object with all share scopes
	 */
	shareScopeMap: "__webpack_require__.S",

	/**
	 * The sharing init sequence function (only runs once per share scope).
	 * Has one argument, the name of the share scope.
	 * Creates a share scope if not existing
	 */
	initializeSharing: "__webpack_require__.I",

	/**
	 * The current scope when getting a module from a remote
	 */
	currentRemoteGetScope: "__webpack_require__.R",

	/**
	 * the filename of the HMR manifest
	 */
	getUpdateManifestFilename: "__webpack_require__.hmrF",

	/**
	 * function downloading the update manifest
	 */
	hmrDownloadManifest: "__webpack_require__.hmrM",

	/**
	 * array with handler functions to download chunk updates
	 */
	hmrDownloadUpdateHandlers: "__webpack_require__.hmrC",

	/**
	 * object with all hmr module data for all modules
	 */
	hmrModuleData: "__webpack_require__.hmrD",

	/**
	 * array with handler functions when a module should be invalidated
	 */
	hmrInvalidateModuleHandlers: "__webpack_require__.hmrI",

	/**
	 * the prefix for storing state of runtime modules when hmr is enabled
	 */
	hmrRuntimeStatePrefix: "__webpack_require__.hmrS",

	/**
	 * the AMD define function
	 */
	amdDefine: "__webpack_require__.amdD",

	/**
	 * the AMD options
	 */
	amdOptions: "__webpack_require__.amdO",

	/**
	 * the System polyfill object
	 */
	system: "__webpack_require__.System",

	/**
	 * the shorthand for Object.prototype.hasOwnProperty
	 * using of it decreases the compiled bundle size
	 */
	hasOwnProperty: "__webpack_require__.o",

	/**
	 * the System.register context object
	 */
	systemContext: "__webpack_require__.y",

	/**
	 * the baseURI of current document
	 */
	baseURI: "__webpack_require__.b",

	/**
	 * a RelativeURL class when relative URLs are used
	 */
	relativeUrl: "__webpack_require__.U",

	/**
	 * Creates an async module. The body function must be a async function.
	 * "module.exports" will be decorated with an AsyncModulePromise.
	 * The body function will be called.
	 * To handle async dependencies correctly do this: "([a, b, c] = await handleDependencies([a, b, c]));".
	 * If "hasAwaitAfterDependencies" is truthy, "handleDependencies()" must be called at the end of the body function.
	 * Signature: function(
	 * module: Module,
	 * body: (handleDependencies: (deps: AsyncModulePromise[]) =\> Promise\<any[]\> & () =\> void,
	 * hasAwaitAfterDependencies?: boolean
	 * ) =\> void
	 */
	asyncModule: "__webpack_require__.a"
} as const;

for (const entry of Object.entries(RuntimeGlobals)) {
	REVERSE_RUNTIME_GLOBALS.set(entry[1], entry[0]);
}
