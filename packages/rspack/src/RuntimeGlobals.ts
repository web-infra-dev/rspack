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
import type { RspackOptionsNormalized } from "./config";

export function __from_binding_runtime_globals(
	runtimeRequirements: JsRuntimeGlobals,
	compilerRuntimeGlobals: Record<string, string>
): Set<string> {
	const res = new Set<string>();

	for (const flag of runtimeRequirements.value) {
		if (flag in compilerRuntimeGlobals) {
			res.add(compilerRuntimeGlobals[flag]);
		} else {
			res.add(flag);
		}
	}

	return res;
}

export function __to_binding_runtime_globals(
	runtimeRequirements: Set<string>,
	compilerRuntimeGlobals: Record<string, string>
): JsRuntimeGlobals {
	const res: JsRuntimeGlobals = {
		value: []
	};
	const reversedCompilerRuntimeGlobals = Object.fromEntries(
		Object.entries(compilerRuntimeGlobals).map(([key, value]) => [value, key])
	);

	for (const flag of Array.from(runtimeRequirements)) {
		const item = reversedCompilerRuntimeGlobals[flag];
		if (typeof item === "string") {
			res.value.push(item);
		} else {
			res.value.push(flag as unknown as string);
		}
	}

	return res;
}

enum RuntimeGlobals {
	/**
	 * the internal require function
	 */
	require,

	/**
	 * access to properties of the internal require function/object
	 */
	requireScope,

	/**
	 * the internal exports object
	 */
	exports,

	/**
	 * top-level this need to be the exports object
	 */
	thisAsExports,

	/**
	 * runtime need to return the exports of the last entry module
	 */
	returnExportsFromRuntime,

	/**
	 * the internal module object
	 */
	module,

	/**
	 * the internal module object
	 */
	moduleId,

	/**
	 * the internal module object
	 */
	moduleLoaded,

	/**
	 * the bundle public path
	 */
	publicPath,

	/**
	 * the module id of the entry point
	 */
	entryModuleId,

	/**
	 * the module cache
	 */
	moduleCache,

	/**
	 * the module functions
	 */
	moduleFactories,

	/**
	 * the module functions, with only write access
	 */
	moduleFactoriesAddOnly,

	/**
	 * the chunk ensure function
	 */
	ensureChunk,

	/**
	 * an object with handlers to ensure a chunk
	 */
	ensureChunkHandlers,

	/**
	 * a runtime requirement if ensureChunkHandlers should include loading of chunk needed for entries
	 */
	ensureChunkIncludeEntries,

	/**
	 * the chunk prefetch function
	 */
	prefetchChunk,

	/**
	 * an object with handlers to prefetch a chunk
	 */
	prefetchChunkHandlers,

	/**
	 * the chunk preload function
	 */
	preloadChunk,

	/**
	 * an object with handlers to preload a chunk
	 */
	preloadChunkHandlers,

	/**
	 * the exported property define getters function
	 */
	definePropertyGetters,

	/**
	 * define compatibility on export
	 */
	makeNamespaceObject,

	/**
	 * create a fake namespace object
	 */
	createFakeNamespaceObject,

	/**
	 * compatibility get default export
	 */
	compatGetDefaultExport,

	/**
	 * ES modules decorator
	 */
	harmonyModuleDecorator,

	/**
	 * node.js module decorator
	 */
	nodeModuleDecorator,

	/**
	 * the webpack hash
	 */
	getFullHash,

	/**
	 * an object containing all installed WebAssembly.Instance export objects keyed by module id
	 */
	wasmInstances,

	/**
	 * instantiate a wasm instance from module exports object, id, hash and importsObject
	 */
	instantiateWasm,

	/**
	 * the uncaught error handler for the webpack runtime
	 */
	uncaughtErrorHandler,

	/**
	 * the script nonce
	 */
	scriptNonce,

	/**
	 * function to load a script tag.
	 * Arguments: (url: string, done: (event) =\> void), key?: string | number, chunkId?: string | number) =\> void
	 * done function is called when loading has finished or timeout occurred.
	 * It will attach to existing script tags with data-webpack == uniqueName + ":" + key or src == url.
	 */
	loadScript,

	/**
	 * function to promote a string to a TrustedScript using webpack's Trusted
	 * Types policy
	 * Arguments: (script: string) =\> TrustedScript
	 */
	createScript,

	/**
	 * function to promote a string to a TrustedScriptURL using webpack's Trusted
	 * Types policy
	 * Arguments: (url: string) =\> TrustedScriptURL
	 */
	createScriptUrl,

	/**
	 * function to return webpack's Trusted Types policy
	 * Arguments: () =\> TrustedTypePolicy
	 */
	getTrustedTypesPolicy,

	/**
	 * a flag when a chunk has a fetch priority
	 */
	hasFetchPriority,

	/**
	 * the chunk name of the chunk with the runtime
	 */
	chunkName,

	/**
	 * the runtime id of the current runtime
	 */
	runtimeId,

	/**
	 * the filename of the script part of the chunk
	 */
	getChunkScriptFilename,

	/**
	 * the filename of the css part of the chunk
	 */
	getChunkCssFilename,

	/**
	 * rspack version
	 * @internal
	 */
	rspackVersion,

	/**
	 * a flag when a module/chunk/tree has css modules
	 */
	hasCssModules,

	/**
	 * rspack unique id
	 * @internal
	 */
	rspackUniqueId,

	/**
	 * the filename of the script part of the hot update chunk
	 */
	getChunkUpdateScriptFilename,

	/**
	 * the filename of the css part of the hot update chunk
	 */
	getChunkUpdateCssFilename,

	/**
	 * startup signal from runtime
	 * This will be called when the runtime chunk has been loaded.
	 */
	startup,

	/**
	 * @deprecated
	 * creating a default startup function with the entry modules
	 */
	startupNoDefault,

	/**
	 * startup signal from runtime but only used to add logic after the startup
	 */
	startupOnlyAfter,

	/**
	 * startup signal from runtime but only used to add sync logic before the startup
	 */
	startupOnlyBefore,

	/**
	 * global callback functions for installing chunks
	 */
	chunkCallback,

	/**
	 * method to startup an entrypoint with needed chunks.
	 * Signature: (moduleId: Id, chunkIds: Id[]) =\> any.
	 * Returns the exports of the module or a Promise
	 */
	startupEntrypoint,

	/**
	 * startup signal from runtime for chunk dependencies
	 */
	startupChunkDependencies,

	/**
	 * register deferred code, which will run when certain
	 * chunks are loaded.
	 * Signature: (chunkIds: Id[], fn: () =\> any, priority: int \>= 0 = 0) =\> any
	 * Returned value will be returned directly when all chunks are already loaded
	 * When (priority & 1) it will wait for all other handlers with lower priority to
	 * be executed before itself is executed
	 */
	onChunksLoaded,

	/**
	 * method to install a chunk that was loaded somehow
	 * Signature: (\{ id, ids, modules, runtime \}) =\> void
	 */
	externalInstallChunk,

	/**
	 * interceptor for module executions
	 */
	interceptModuleExecution,

	/**
	 * the global object
	 */
	global,

	/**
	 * an object with all share scopes
	 */
	shareScopeMap,

	/**
	 * The sharing init sequence function (only runs once per share scope).
	 * Has one argument, the name of the share scope.
	 * Creates a share scope if not existing
	 */
	initializeSharing,

	/**
	 * The current scope when getting a module from a remote
	 */
	currentRemoteGetScope,

	/**
	 * the filename of the HMR manifest
	 */
	getUpdateManifestFilename,

	/**
	 * function downloading the update manifest
	 */
	hmrDownloadManifest,

	/**
	 * array with handler functions to download chunk updates
	 */
	hmrDownloadUpdateHandlers,

	/**
	 * object with all hmr module data for all modules
	 */
	hmrModuleData,

	/**
	 * array with handler functions when a module should be invalidated
	 */
	hmrInvalidateModuleHandlers,

	/**
	 * the prefix for storing state of runtime modules when hmr is enabled
	 */
	hmrRuntimeStatePrefix,

	/**
	 * the AMD define function
	 */
	amdDefine,

	/**
	 * the AMD options
	 */
	amdOptions,

	/**
	 * the System polyfill object
	 */
	system,

	/**
	 * the shorthand for Object.prototype.hasOwnProperty
	 * using of it decreases the compiled bundle size
	 */
	hasOwnProperty,

	/**
	 * the System.register context object
	 */
	systemContext,

	/**
	 * the baseURI of current document
	 */
	baseURI,

	/**
	 * a RelativeURL class when relative URLs are used
	 */
	relativeUrl,

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
	asyncModule,

	asyncModuleExportSymbol,

	makeDeferredNamespaceObject,

	makeDeferredNamespaceObjectSymbol
}

export const isReservedRuntimeGlobal = (
	r: string,
	compilerRuntimeGlobals: Record<string, string>
) => Object.values(compilerRuntimeGlobals).includes(r);

export function renderModulePrefix(
	_compilerOptions: RspackOptionsNormalized
): string {
	return "webpack/runtime/";
}

export enum RuntimeVariable {
	Require,
	Modules,
	ModuleCache,
	Module,
	Exports,
	StartupExec
}

export function renderRuntimeVariables(
	variable: RuntimeVariable,
	_compilerOptions?: RspackOptionsNormalized
): string {
	switch (variable) {
		case RuntimeVariable.Require:
			return "__webpack_require__";
		case RuntimeVariable.Modules:
			return "__webpack_modules__";
		case RuntimeVariable.ModuleCache:
			return "__webpack_module_cache__";
		case RuntimeVariable.Module:
			return "__webpack_module__";
		case RuntimeVariable.Exports:
			return "__webpack_exports__";
		case RuntimeVariable.StartupExec:
			return "__webpack_exec__";
	}
}

function renderRuntimeGlobals(
	runtimeGlobals: RuntimeGlobals,
	_compilerOptions?: RspackOptionsNormalized
): string {
	const scope_name = renderRuntimeVariables(
		RuntimeVariable.Require,
		_compilerOptions
	);
	const exports_name = renderRuntimeVariables(
		RuntimeVariable.Exports,
		_compilerOptions
	);
	switch (runtimeGlobals) {
		case RuntimeGlobals.require:
			return scope_name;
		case RuntimeGlobals.requireScope:
			return `${scope_name}.*`;
		case RuntimeGlobals.exports:
			return exports_name;
		case RuntimeGlobals.thisAsExports:
			return `top-level-this-exports`;
		case RuntimeGlobals.returnExportsFromRuntime:
			return `return-exports-from-runtime`;
		case RuntimeGlobals.module:
			return `module`;
		case RuntimeGlobals.moduleId:
			return `module.id`;
		case RuntimeGlobals.moduleLoaded:
			return `module.loaded`;
		case RuntimeGlobals.publicPath:
			return `${scope_name}.p`;
		case RuntimeGlobals.entryModuleId:
			return `${scope_name}.s`;
		case RuntimeGlobals.moduleCache:
			return `${scope_name}.c`;
		case RuntimeGlobals.moduleFactories:
			return `${scope_name}.m`;
		case RuntimeGlobals.moduleFactoriesAddOnly:
			return `${scope_name}.m (add only)`;
		case RuntimeGlobals.ensureChunk:
			return `${scope_name}.e`;
		case RuntimeGlobals.ensureChunkHandlers:
			return `${scope_name}.f`;
		case RuntimeGlobals.ensureChunkIncludeEntries:
			return `${scope_name}.f (include entries)`;
		case RuntimeGlobals.prefetchChunk:
			return `${scope_name}.E`;
		case RuntimeGlobals.prefetchChunkHandlers:
			return `${scope_name}.F`;
		case RuntimeGlobals.preloadChunk:
			return `${scope_name}.G`;
		case RuntimeGlobals.preloadChunkHandlers:
			return `${scope_name}.H`;
		case RuntimeGlobals.definePropertyGetters:
			return `${scope_name}.d`;
		case RuntimeGlobals.makeNamespaceObject:
			return `${scope_name}.r`;
		case RuntimeGlobals.createFakeNamespaceObject:
			return `${scope_name}.t`;
		case RuntimeGlobals.compatGetDefaultExport:
			return `${scope_name}.n`;
		case RuntimeGlobals.harmonyModuleDecorator:
			return `${scope_name}.hmd`;
		case RuntimeGlobals.nodeModuleDecorator:
			return `${scope_name}.nmd`;
		case RuntimeGlobals.getFullHash:
			return `${scope_name}.h`;
		case RuntimeGlobals.wasmInstances:
			return `${scope_name}.w`;
		case RuntimeGlobals.instantiateWasm:
			return `${scope_name}.v`;
		case RuntimeGlobals.uncaughtErrorHandler:
			return `${scope_name}.oe`;
		case RuntimeGlobals.scriptNonce:
			return `${scope_name}.nc`;
		case RuntimeGlobals.loadScript:
			return `${scope_name}.l`;
		case RuntimeGlobals.createScript:
			return `${scope_name}.ts`;
		case RuntimeGlobals.createScriptUrl:
			return `${scope_name}.tu`;
		case RuntimeGlobals.getTrustedTypesPolicy:
			return `${scope_name}.tt`;
		case RuntimeGlobals.hasFetchPriority:
			return `has fetch priority`;
		case RuntimeGlobals.chunkName:
			return `${scope_name}.cn`;
		case RuntimeGlobals.runtimeId:
			return `${scope_name}.j`;
		case RuntimeGlobals.getChunkScriptFilename:
			return `${scope_name}.u`;
		case RuntimeGlobals.getChunkCssFilename:
			return `${scope_name}.k`;
		case RuntimeGlobals.rspackVersion:
			return `${scope_name}.rv`;
		case RuntimeGlobals.hasCssModules:
			return `has css modules`;
		case RuntimeGlobals.rspackUniqueId:
			return `${scope_name}.ruid`;
		case RuntimeGlobals.getChunkUpdateScriptFilename:
			return `${scope_name}.hu`;
		case RuntimeGlobals.getChunkUpdateCssFilename:
			return `${scope_name}.hk`;
		case RuntimeGlobals.startup:
			return `${scope_name}.x`;
		case RuntimeGlobals.startupNoDefault:
			return `${scope_name}.x (no default handler)`;
		case RuntimeGlobals.startupOnlyAfter:
			return `${scope_name}.x (only after)`;
		case RuntimeGlobals.startupOnlyBefore:
			return `${scope_name}.x (only before)`;
		case RuntimeGlobals.chunkCallback:
			return `global chunk callback`;
		case RuntimeGlobals.startupEntrypoint:
			return `${scope_name}.X`;
		case RuntimeGlobals.startupChunkDependencies:
			return `${scope_name}.x (chunk dependencies)`;
		case RuntimeGlobals.onChunksLoaded:
			return `${scope_name}.O`;
		case RuntimeGlobals.externalInstallChunk:
			return `${scope_name}.C`;
		case RuntimeGlobals.interceptModuleExecution:
			return `${scope_name}.i`;
		case RuntimeGlobals.global:
			return `${scope_name}.g`;
		case RuntimeGlobals.shareScopeMap:
			return `${scope_name}.S`;
		case RuntimeGlobals.initializeSharing:
			return `${scope_name}.I`;
		case RuntimeGlobals.currentRemoteGetScope:
			return `${scope_name}.R`;
		case RuntimeGlobals.getUpdateManifestFilename:
			return `${scope_name}.hmrF`;
		case RuntimeGlobals.hmrDownloadManifest:
			return `${scope_name}.hmrM`;
		case RuntimeGlobals.hmrDownloadUpdateHandlers:
			return `${scope_name}.hmrC`;
		case RuntimeGlobals.hmrModuleData:
			return `${scope_name}.hmrD`;
		case RuntimeGlobals.hmrInvalidateModuleHandlers:
			return `${scope_name}.hmrI`;
		case RuntimeGlobals.hmrRuntimeStatePrefix:
			return `${scope_name}.hmrS`;
		case RuntimeGlobals.amdDefine:
			return `${scope_name}.amdD`;
		case RuntimeGlobals.amdOptions:
			return `${scope_name}.amdO`;
		case RuntimeGlobals.system:
			return `${scope_name}.System`;
		case RuntimeGlobals.hasOwnProperty:
			return `${scope_name}.o`;
		case RuntimeGlobals.systemContext:
			return `${scope_name}.y`;
		case RuntimeGlobals.baseURI:
			return `${scope_name}.b`;
		case RuntimeGlobals.relativeUrl:
			return `${scope_name}.U`;
		case RuntimeGlobals.asyncModule:
			return `${scope_name}.a`;
		case RuntimeGlobals.asyncModuleExportSymbol:
			return `${scope_name}.aE`;
		case RuntimeGlobals.makeDeferredNamespaceObject:
			return `${scope_name}.z`;
		case RuntimeGlobals.makeDeferredNamespaceObjectSymbol:
			return `${scope_name}.zS`;
		default:
			return "";
	}
}

export function createCompilerRuntimeGlobals(
	compilerOptions?: RspackOptionsNormalized
): Record<keyof typeof RuntimeGlobals, string> {
	const res: Record<string, string> = {};
	for (const key of Object.keys(RuntimeGlobals)) {
		res[key] = renderRuntimeGlobals(
			RuntimeGlobals[key as keyof typeof RuntimeGlobals],
			compilerOptions
		);
	}
	return res as unknown as Record<keyof typeof RuntimeGlobals, string>;
}

const DefaultRuntimeGlobals = createCompilerRuntimeGlobals();

export { DefaultRuntimeGlobals as RuntimeGlobals };
