/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Compiler.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import type * as binding from "@rspack/binding";
import * as liteTapable from "@rspack/lite-tapable";

import ExecuteModulePlugin from "./ExecuteModulePlugin";
import ConcurrentCompilationError from "./error/ConcurrentCompilationError";
import Cache from "./lib/Cache";
import CacheFacade from "./lib/CacheFacade";

import type { Chunk } from "./Chunk";
import { Compilation } from "./Compilation";
import { ContextModuleFactory } from "./ContextModuleFactory";
import {
	ThreadsafeIntermediateNodeFS,
	ThreadsafeOutputNodeFS
} from "./FileSystem";
import { NormalModuleFactory } from "./NormalModuleFactory";
import { ResolverFactory } from "./ResolverFactory";
import { RuleSetCompiler } from "./RuleSetCompiler";
import {
	__from_binding_runtime_globals,
	__to_binding_runtime_globals
} from "./RuntimeGlobals";
import { Stats } from "./Stats";
import { Watching } from "./Watching";
import {
	JsLoaderRspackPlugin,
	createRsdoctorPluginHooksRegisters,
	createRuntimePluginHooksRegisters
} from "./builtin-plugin";
import { getRawOptions } from "./config";
import { rspack } from "./index";
import { unsupported } from "./util";

import { canInherentFromParent } from "./builtin-plugin/base";
import { applyRspackOptionsDefaults, getPnpDefault } from "./config/defaults";
import { Logger } from "./logging/Logger";
import { assertNotNill } from "./util/assertNotNil";
import { checkVersion } from "./util/bindingVersionCheck";
import { makePathsRelative } from "./util/identifier";

import type Watchpack from "watchpack";
import type { Source } from "webpack-sources";
import type { CompilationParams } from "./Compilation";
import type { FileSystemInfoEntry } from "./FileSystemInfo";
import type {
	EntryNormalized,
	OutputNormalized,
	RspackOptionsNormalized,
	RspackPluginInstance,
	WatchOptions
} from "./config";
import {
	createCompilationHooksRegisters,
	createCompilerHooksRegisters,
	createContextModuleFactoryHooksRegisters,
	createHtmlPluginHooksRegisters,
	createJavaScriptModulesHooksRegisters,
	createNormalModuleFactoryHooksRegisters
} from "./taps";
import type {
	InputFileSystem,
	IntermediateFileSystem,
	OutputFileSystem,
	WatchFileSystem
} from "./util/fs";

export interface AssetEmittedInfo {
	content: Buffer;
	source: Source;
	outputPath: string;
	targetPath: string;
	compilation: Compilation;
}

const COMPILATION_WEAK_MAP = new WeakMap<binding.JsCompilation, Compilation>();

class Compiler {
	#instance?: binding.JsCompiler;
	#initial: boolean;

	#compilation?: Compilation;
	#compilationParams?: CompilationParams;

	#builtinPlugins: binding.BuiltinPlugin[];

	#moduleExecutionResultsMap: Map<number, any>;

	#nonSkippableRegisters: binding.RegisterJsTapKind[];
	#registers?: binding.RegisterJsTaps;

	#ruleSet: RuleSetCompiler;

	hooks: {
		done: liteTapable.AsyncSeriesHook<Stats>;
		afterDone: liteTapable.SyncHook<Stats>;
		thisCompilation: liteTapable.SyncHook<[Compilation, CompilationParams]>;
		compilation: liteTapable.SyncHook<[Compilation, CompilationParams]>;
		invalid: liteTapable.SyncHook<[string | null, number]>;
		compile: liteTapable.SyncHook<[CompilationParams]>;
		normalModuleFactory: liteTapable.SyncHook<NormalModuleFactory>;
		contextModuleFactory: liteTapable.SyncHook<ContextModuleFactory>;
		initialize: liteTapable.SyncHook<[]>;
		shouldEmit: liteTapable.SyncBailHook<[Compilation], boolean>;
		infrastructureLog: liteTapable.SyncBailHook<[string, string, any[]], true>;
		beforeRun: liteTapable.AsyncSeriesHook<[Compiler]>;
		run: liteTapable.AsyncSeriesHook<[Compiler]>;
		emit: liteTapable.AsyncSeriesHook<[Compilation]>;
		assetEmitted: liteTapable.AsyncSeriesHook<[string, AssetEmittedInfo]>;
		afterEmit: liteTapable.AsyncSeriesHook<[Compilation]>;
		failed: liteTapable.SyncHook<[Error]>;
		shutdown: liteTapable.AsyncSeriesHook<[]>;
		watchRun: liteTapable.AsyncSeriesHook<[Compiler]>;
		watchClose: liteTapable.SyncHook<[]>;
		environment: liteTapable.SyncHook<[]>;
		afterEnvironment: liteTapable.SyncHook<[]>;
		afterPlugins: liteTapable.SyncHook<[Compiler]>;
		afterResolvers: liteTapable.SyncHook<[Compiler]>;
		make: liteTapable.AsyncParallelHook<[Compilation]>;
		beforeCompile: liteTapable.AsyncSeriesHook<[CompilationParams]>;
		afterCompile: liteTapable.AsyncSeriesHook<[Compilation]>;
		finishMake: liteTapable.AsyncSeriesHook<[Compilation]>;
		entryOption: liteTapable.SyncBailHook<[string, EntryNormalized], any>;
		additionalPass: liteTapable.AsyncSeriesHook<[]>;
	};

	webpack: typeof rspack;
	rspack: typeof rspack;
	name?: string;
	parentCompilation?: Compilation;
	root: Compiler;
	outputPath: string;

	running: boolean;
	idle: boolean;
	resolverFactory: ResolverFactory;
	infrastructureLogger: any;
	watching?: Watching;

	inputFileSystem: InputFileSystem | null;
	intermediateFileSystem: IntermediateFileSystem | null;
	outputFileSystem: OutputFileSystem | null;
	watchFileSystem: WatchFileSystem | null;

	records: Record<string, any[]>;
	modifiedFiles?: ReadonlySet<string>;
	removedFiles?: ReadonlySet<string>;
	fileTimestamps?: ReadonlyMap<string, FileSystemInfoEntry | "ignore" | null>;
	contextTimestamps?: ReadonlyMap<
		string,
		FileSystemInfoEntry | "ignore" | null
	>;
	fsStartTime?: number;

	watchMode: boolean;
	context: string;
	cache: Cache;
	compilerPath: string;
	options: RspackOptionsNormalized;

	constructor(context: string, options: RspackOptionsNormalized) {
		this.#initial = true;

		this.#builtinPlugins = [];

		this.#nonSkippableRegisters = [];
		this.#moduleExecutionResultsMap = new Map();

		this.#ruleSet = new RuleSetCompiler();

		this.hooks = {
			initialize: new liteTapable.SyncHook([]),
			shouldEmit: new liteTapable.SyncBailHook(["compilation"]),
			done: new liteTapable.AsyncSeriesHook<Stats>(["stats"]),
			afterDone: new liteTapable.SyncHook<Stats>(["stats"]),
			beforeRun: new liteTapable.AsyncSeriesHook(["compiler"]),
			run: new liteTapable.AsyncSeriesHook(["compiler"]),
			emit: new liteTapable.AsyncSeriesHook(["compilation"]),
			assetEmitted: new liteTapable.AsyncSeriesHook(["file", "info"]),
			afterEmit: new liteTapable.AsyncSeriesHook(["compilation"]),
			thisCompilation: new liteTapable.SyncHook<
				[Compilation, CompilationParams]
			>(["compilation", "params"]),
			compilation: new liteTapable.SyncHook<[Compilation, CompilationParams]>([
				"compilation",
				"params"
			]),
			invalid: new liteTapable.SyncHook(["filename", "changeTime"]),
			compile: new liteTapable.SyncHook(["params"]),
			infrastructureLog: new liteTapable.SyncBailHook([
				"origin",
				"type",
				"args"
			]),
			failed: new liteTapable.SyncHook(["error"]),
			shutdown: new liteTapable.AsyncSeriesHook([]),
			normalModuleFactory: new liteTapable.SyncHook<NormalModuleFactory>([
				"normalModuleFactory"
			]),
			contextModuleFactory: new liteTapable.SyncHook<ContextModuleFactory>([
				"contextModuleFactory"
			]),
			watchRun: new liteTapable.AsyncSeriesHook(["compiler"]),
			watchClose: new liteTapable.SyncHook([]),
			environment: new liteTapable.SyncHook([]),
			afterEnvironment: new liteTapable.SyncHook([]),
			afterPlugins: new liteTapable.SyncHook(["compiler"]),
			afterResolvers: new liteTapable.SyncHook(["compiler"]),
			make: new liteTapable.AsyncParallelHook(["compilation"]),
			beforeCompile: new liteTapable.AsyncSeriesHook(["params"]),
			afterCompile: new liteTapable.AsyncSeriesHook(["compilation"]),
			finishMake: new liteTapable.AsyncSeriesHook(["compilation"]),
			entryOption: new liteTapable.SyncBailHook(["context", "entry"]),
			additionalPass: new liteTapable.AsyncSeriesHook([])
		};

		this.webpack = rspack;
		this.rspack = rspack;
		this.root = this;
		this.outputPath = "";

		this.inputFileSystem = null;
		this.intermediateFileSystem = null;
		this.outputFileSystem = null;
		this.watchFileSystem = null;

		this.records = {};

		this.options = options;
		this.context = context;
		this.cache = new Cache();

		this.compilerPath = "";

		this.running = false;

		this.idle = false;

		this.watchMode = false;
		// this is a bit tricky since applyDefaultOptions is executed later, so we don't get the `resolve.pnp` default value
		// we need to call pnp defaultValue early here
		this.resolverFactory = new ResolverFactory(
			options.resolve.pnp ?? getPnpDefault()
		);
		new JsLoaderRspackPlugin(this).apply(this);
		new ExecuteModulePlugin().apply(this);

		// this.hooks.shutdown.tap("rspack:cleanup", () => {
		// 	// Delayed rspack cleanup to the next tick.
		// 	// This supports calls to `fn rspack` to do something with `Stats` within the same tick.
		// 	process.nextTick(() => {
		// 		if (!this.running) {
		// 			this.#instance = undefined;
		// 			this.#compilation && (this.#compilation.__internal__shutdown = true);
		// 		}
		// 	});
		// });
	}

	get recordsInputPath() {
		return unsupported("Compiler.recordsInputPath");
	}

	get recordsOutputPath() {
		return unsupported("Compiler.recordsOutputPath");
	}

	get managedPaths() {
		return unsupported("Compiler.managedPaths");
	}

	get immutablePaths() {
		return unsupported("Compiler.immutablePaths");
	}

	get _lastCompilation() {
		return this.#compilation;
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 * @internal
	 */
	get __internal__builtinPlugins() {
		return this.#builtinPlugins;
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 * @internal
	 */
	get __internal__ruleSet() {
		return this.#ruleSet;
	}

	/**
	 * @param name - cache name
	 * @returns the cache facade instance
	 */
	getCache(name: string): CacheFacade {
		return new CacheFacade(
			this.cache,
			`${this.compilerPath}${name}`,
			this.options.output.hashFunction as string
		);
	}

	/**
	 * @param name - name of the logger, or function called once to get the logger name
	 * @returns a logger with that name
	 */
	getInfrastructureLogger(name: string | (() => string)) {
		if (!name) {
			throw new TypeError(
				"Compiler.getInfrastructureLogger(name) called without a name"
			);
		}

		let normalizedName = name;
		return new Logger(
			(type, args) => {
				if (typeof normalizedName === "function") {
					normalizedName = normalizedName();
					if (!normalizedName) {
						throw new TypeError(
							"Compiler.getInfrastructureLogger(name) called with a function not returning a name"
						);
					}
				} else {
					if (
						this.hooks.infrastructureLog.call(normalizedName, type, args) ===
						undefined
					) {
						if (this.infrastructureLogger !== undefined) {
							this.infrastructureLogger(normalizedName, type, args);
						}
					}
				}
			},
			(childName): any => {
				let normalizedChildName = childName;
				if (typeof normalizedName === "function") {
					if (typeof normalizedChildName === "function") {
						return this.getInfrastructureLogger(() => {
							if (typeof normalizedName === "function") {
								normalizedName = normalizedName();
								if (!normalizedName) {
									throw new TypeError(
										"Compiler.getInfrastructureLogger(name) called with a function not returning a name"
									);
								}
							}
							if (typeof normalizedChildName === "function") {
								normalizedChildName = normalizedChildName();
								if (!normalizedChildName) {
									throw new TypeError(
										"Logger.getChildLogger(name) called with a function not returning a name"
									);
								}
							}
							return `${normalizedName}/${normalizedChildName}`;
						});
					}
					return this.getInfrastructureLogger(() => {
						if (typeof normalizedName === "function") {
							normalizedName = normalizedName();
							if (!normalizedName) {
								throw new TypeError(
									"Compiler.getInfrastructureLogger(name) called with a function not returning a name"
								);
							}
						}
						return `${normalizedName}/${normalizedChildName}`;
					});
				}
				if (typeof normalizedChildName === "function") {
					return this.getInfrastructureLogger(() => {
						if (typeof normalizedChildName === "function") {
							normalizedChildName = normalizedChildName();
							if (!normalizedChildName) {
								throw new TypeError(
									"Logger.getChildLogger(name) called with a function not returning a name"
								);
							}
						}
						return `${normalizedName}/${normalizedChildName}`;
					});
				}
				return this.getInfrastructureLogger(
					`${normalizedName}/${normalizedChildName}`
				);
			}
		);
	}

	/**
	 * @param watchOptions - the watcher's options
	 * @param handler - signals when the call finishes
	 * @returns a compiler watcher
	 */
	watch(
		watchOptions: Watchpack.WatchOptions,
		handler: liteTapable.Callback<Error, Stats>
	): Watching {
		if (this.running) {
			// cannot be resolved without assertion
			// copy from webpack
			// Type 'void' is not assignable to type 'Watching'.
			return handler(new ConcurrentCompilationError()) as unknown as Watching;
		}
		this.running = true;
		this.watchMode = true;
		this.watching = new Watching(this, watchOptions as WatchOptions, handler);
		return this.watching;
	}

	/**
	 * @param callback - signals when the call finishes
	 * @param options - additional data like modifiedFiles, removedFiles
	 */
	run(
		callback: liteTapable.Callback<Error, Stats>,
		options: {
			modifiedFiles?: ReadonlySet<string>;
			removedFiles?: ReadonlySet<string>;
		} = {}
	) {
		if (this.running) {
			return callback(new ConcurrentCompilationError());
		}

		this.modifiedFiles = options.modifiedFiles;
		this.removedFiles = options.removedFiles;
		const startTime = Date.now();
		this.running = true;

		const finalCallback = (err: Error | null, stats?: Stats) => {
			this.idle = true;
			this.cache.beginIdle();
			this.idle = true;
			this.running = false;
			if (err) {
				this.hooks.failed.call(err);
			}
			if (callback) {
				callback(err, stats);
			}
			this.hooks.afterDone.call(stats!);
		};

		const onCompiled = (
			err: Error | null,
			_compilation: Compilation | undefined
		) => {
			if (err) {
				return finalCallback(err);
			}

			const compilation = _compilation!;

			if (compilation.hooks.needAdditionalPass.call()) {
				compilation.needAdditionalPass = true;

				compilation.startTime = startTime;
				compilation.endTime = Date.now();
				const stats = new Stats(compilation);
				this.hooks.done.callAsync(stats, err => {
					if (err) {
						return finalCallback(err);
					}

					this.hooks.additionalPass.callAsync(err => {
						if (err) return finalCallback(err);
						this.compile(onCompiled);
					});
				});
				return;
			}

			compilation.startTime = startTime;
			compilation.endTime = Date.now();
			const stats = new Stats(compilation);
			this.hooks.done.callAsync(stats, err => {
				if (err) {
					return finalCallback(err);
				}
				return finalCallback(null, stats);
			});
		};

		const run = () => {
			this.hooks.beforeRun.callAsync(this, err => {
				if (err) {
					return finalCallback(err);
				}
				this.hooks.run.callAsync(this, err => {
					if (err) {
						return finalCallback(err);
					}
					this.compile(onCompiled);
				});
			});
		};

		if (this.idle) {
			this.cache.endIdle(err => {
				if (err) return callback(err);
				this.idle = false;
				run();
			});
		} else {
			run();
		}
	}

	runAsChild(
		callback: (
			err?: null | Error,
			entries?: Chunk[],
			compilation?: Compilation
		) => any
	) {
		const finalCallback = (
			err: Error | null,
			entries?: Chunk[],
			compilation?: Compilation
		) => {
			try {
				callback(err, entries, compilation);
			} catch (e) {
				const err = new Error(`compiler.runAsChild callback error: ${e}`);
				// err.details = e.stack;
				this.parentCompilation!.errors.push(err);
				// TODO: remove once this works
				console.log(e);
			}
		};

		this.compile((err, compilation) => {
			if (err) {
				return finalCallback(err);
			}

			assertNotNill(compilation);

			this.parentCompilation!.children.push(compilation);
			for (const { name, source, info } of compilation.getAssets()) {
				// Do not emit asset if source is not available.
				// Webpack will emit it anyway.
				if (source) {
					this.parentCompilation!.emitAsset(name, source, info);
				}
			}

			const entries = [];
			for (const ep of compilation.entrypoints.values()) {
				entries.push(...ep.chunks);
			}

			return finalCallback(null, entries, compilation);
		});
	}

	purgeInputFileSystem() {
		this.inputFileSystem?.purge?.();
	}

	/**
	 * @param compilation - the compilation
	 * @param compilerName - the compiler's name
	 * @param compilerIndex - the compiler's index
	 * @param outputOptions - the output options
	 * @param plugins - the plugins to apply
	 * @returns a child compiler
	 */
	createChildCompiler(
		compilation: Compilation,
		compilerName: string,
		compilerIndex: number,
		outputOptions: OutputNormalized,
		plugins: RspackPluginInstance[]
	): Compiler {
		const options: RspackOptionsNormalized = {
			...this.options,
			output: {
				...this.options.output,
				...outputOptions
			}
		};
		applyRspackOptionsDefaults(options);
		const childCompiler = new Compiler(this.context, options);
		childCompiler.name = compilerName;
		childCompiler.outputPath = this.outputPath;
		childCompiler.inputFileSystem = this.inputFileSystem;
		childCompiler.outputFileSystem = null;
		childCompiler.modifiedFiles = this.modifiedFiles;
		childCompiler.removedFiles = this.removedFiles;
		childCompiler.fileTimestamps = this.fileTimestamps;
		childCompiler.contextTimestamps = this.contextTimestamps;
		childCompiler.fsStartTime = this.fsStartTime;
		childCompiler.cache = this.cache;
		childCompiler.compilerPath = `${this.compilerPath}${compilerName}|${compilerIndex}|`;

		const relativeCompilerName = makePathsRelative(
			this.context,
			compilerName,
			this.root
		);
		if (!this.records[relativeCompilerName]) {
			this.records[relativeCompilerName] = [];
		}
		if (this.records[relativeCompilerName][compilerIndex]) {
			childCompiler.records = this.records[relativeCompilerName][compilerIndex];
		} else {
			this.records[relativeCompilerName].push((childCompiler.records = {}));
		}

		childCompiler.parentCompilation = compilation;
		childCompiler.root = this.root;
		if (Array.isArray(plugins)) {
			for (const plugin of plugins) {
				if (plugin) {
					plugin.apply(childCompiler);
				}
			}
		}

		childCompiler.#builtinPlugins = [
			...childCompiler.#builtinPlugins,
			...this.#builtinPlugins.filter(
				plugin => plugin.canInherentFromParent === true
			)
		];

		for (const hookName in this.hooks) {
			type HookNames = keyof Compiler["hooks"];

			const name = hookName as unknown as HookNames;

			if (canInherentFromParent(name)) {
				if (childCompiler.hooks[name]) {
					childCompiler.hooks[name].taps = this.hooks[name].taps.slice();
				}
			}
		}

		compilation.hooks.childCompiler.call(
			childCompiler,
			compilerName,
			compilerIndex
		);

		return childCompiler;
	}

	isChild() {
		const isRoot = this.root === this;
		return !isRoot;
	}

	/**
	 * Create a compilation and run it, which is the basic method that `compiler.run` and `compiler.watch` depend on.
	 * TODO: make this method private in the next major release
	 * @private this method is only used in Rspack core
	 */
	compile(callback: liteTapable.Callback<Error, Compilation>) {
		const startTime = Date.now();
		const params = this.#newCompilationParams();
		this.hooks.beforeCompile.callAsync(params, (err: any) => {
			if (err) {
				return callback(err);
			}
			this.hooks.compile.call(params);
			this.#resetThisCompilation();

			this.#build(err => {
				if (err) {
					return callback(err);
				}
				this.#compilation!.startTime = startTime;
				this.#compilation!.endTime = Date.now();
				this.hooks.afterCompile.callAsync(this.#compilation!, err => {
					if (err) {
						return callback(err);
					}
					return callback(null, this.#compilation);
				});
			});
		});
	}

	close(callback: (error?: Error | null) => void) {
		if (this.watching) {
			// When there is still an active watching, close this #initial
			this.watching.close(() => {
				this.close(callback);
			});
			return;
		}
		this.hooks.shutdown.callAsync(err => {
			if (err) return callback(err);
			this.cache.shutdown(callback);
		});
	}

	#build(callback: (error: Error | null) => void) {
		this.#getInstance((error, instance) => {
			if (error) {
				return callback(error);
			}
			if (!this.#initial) {
				instance!.rebuild(
					Array.from(this.modifiedFiles || []),
					Array.from(this.removedFiles || []),
					callback
				);
				return;
			}
			this.#initial = false;
			instance!.build(callback);
		});
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 * @internal
	 */
	__internal__rebuild(
		modifiedFiles?: ReadonlySet<string>,
		removedFiles?: ReadonlySet<string>,
		callback?: (error: Error | null) => void
	) {
		this.#getInstance((error, instance) => {
			if (error) {
				return callback?.(error);
			}
			instance!.rebuild(
				Array.from(modifiedFiles || []),
				Array.from(removedFiles || []),
				error => {
					if (error) {
						return callback?.(error);
					}
					callback?.(null);
				}
			);
		});
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 * @internal
	 */
	__internal__create_compilation(native: binding.JsCompilation): Compilation {
		let compilation = COMPILATION_WEAK_MAP.get(native);

		if (!compilation) {
			compilation = new Compilation(this, native);
			compilation.name = this.name;
			COMPILATION_WEAK_MAP.set(native, compilation);
		}

		this.#compilation = compilation;
		return compilation;
	}

	#resetThisCompilation() {
		// reassign new compilation in thisCompilation
		this.#compilation = undefined;
		// ensure thisCompilation must call
		this.hooks.thisCompilation.intercept({
			call: () => {}
		});
	}

	#newCompilationParams(): CompilationParams {
		const normalModuleFactory = new NormalModuleFactory(this.resolverFactory);
		this.hooks.normalModuleFactory.call(normalModuleFactory);
		const contextModuleFactory = new ContextModuleFactory();
		this.hooks.contextModuleFactory.call(contextModuleFactory);
		const params = {
			normalModuleFactory,
			contextModuleFactory
		};
		this.#compilationParams = params;
		return params;
	}

	/**
	 * Lazy initialize instance so it could access the changed options
	 */
	#getInstance(
		callback: (error: Error | null, instance?: binding.JsCompiler) => void
	): void {
		const error = checkVersion();
		if (error) {
			return callback(error);
		}

		if (this.#instance) {
			return callback(null, this.#instance);
		}

		const options = this.options;
		const rawOptions = getRawOptions(options, this);
		rawOptions.__references = Object.fromEntries(
			this.#ruleSet.builtinReferences.entries()
		);

		const instanceBinding: typeof binding = require("@rspack/binding");

		this.#registers = this.#createHooksRegisters();

		this.#instance = new instanceBinding.JsCompiler(
			this.compilerPath,
			rawOptions,
			this.#builtinPlugins,
			this.#registers,
			ThreadsafeOutputNodeFS.__to_binding(this.outputFileSystem!),
			this.intermediateFileSystem
				? ThreadsafeIntermediateNodeFS.__to_binding(this.intermediateFileSystem)
				: undefined,
			ResolverFactory.__to_binding(this.resolverFactory)
		);

		callback(null, this.#instance);
	}

	#createHooksRegisters(): binding.RegisterJsTaps {
		const ref = new WeakRef(this);
		const getCompiler = () => ref.deref()!;
		const createTap = this.#createHookRegisterTaps.bind(this);
		const createMapTap = this.#createHookMapRegisterTaps.bind(this);
		return {
			...createCompilerHooksRegisters(getCompiler, createTap, createMapTap),
			...createCompilationHooksRegisters(getCompiler, createTap, createMapTap),
			...createNormalModuleFactoryHooksRegisters(
				getCompiler,
				createTap,
				createMapTap
			),
			...createContextModuleFactoryHooksRegisters(
				getCompiler,
				createTap,
				createMapTap
			),
			...createJavaScriptModulesHooksRegisters(
				getCompiler,
				createTap,
				createMapTap
			),
			...createHtmlPluginHooksRegisters(getCompiler, createTap, createMapTap),
			...createRuntimePluginHooksRegisters(
				getCompiler,
				createTap,
				createMapTap
			),
			...createRsdoctorPluginHooksRegisters(
				getCompiler,
				createTap,
				createMapTap
			)
		};
	}

	#updateNonSkippableRegisters() {
		const kinds: binding.RegisterJsTapKind[] = [];
		for (const { getHook, getHookMap, registerKind } of Object.values(
			this.#registers!
		)) {
			const get = getHook ?? getHookMap;
			const hookOrMap = get();
			if (hookOrMap.isUsed()) {
				kinds.push(registerKind);
			}
		}
		if (this.#nonSkippableRegisters.join() !== kinds.join()) {
			this.#getInstance((_error, instance) => {
				instance!.setNonSkippableRegisters(kinds);
				this.#nonSkippableRegisters = kinds;
			});
		}
	}

	#decorateJsTaps(jsTaps: binding.JsTap[]) {
		if (jsTaps.length > 0) {
			const last = jsTaps[jsTaps.length - 1];
			const old = last.function;
			last.function = (...args: any[]) => {
				const result = old(...args);
				if (result && typeof result.then === "function") {
					return result.then((r: any) => {
						this.#updateNonSkippableRegisters();
						return r;
					});
				}
				this.#updateNonSkippableRegisters();
				return result;
			};
		}
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 * @internal
	 */
	#createHookRegisterTaps<T, R, A>(
		registerKind: binding.RegisterJsTapKind,
		getHook: () => liteTapable.Hook<T, R, A>,
		createTap: (queried: liteTapable.QueriedHook<T, R, A>) => any
	): (stages: number[]) => binding.JsTap[] {
		const that = new WeakRef(this);
		const getTaps = (stages: number[]) => {
			const compiler = that.deref()!;
			const hook = getHook();
			if (!hook.isUsed()) return [];
			const breakpoints = [
				liteTapable.minStage,
				...stages,
				liteTapable.maxStage
			];
			const jsTaps: binding.JsTap[] = [];
			for (let i = 0; i < breakpoints.length - 1; i++) {
				const from = breakpoints[i];
				const to = breakpoints[i + 1];
				const stageRange = [from, to] as const;
				const queried = hook.queryStageRange(stageRange);
				if (!queried.isUsed()) continue;
				jsTaps.push({
					function: createTap(queried),
					stage: liteTapable.safeStage(from + 1)
				});
			}
			compiler.#decorateJsTaps(jsTaps);
			return jsTaps;
		};
		getTaps.registerKind = registerKind;
		getTaps.getHook = getHook;
		return getTaps;
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 * @internal
	 */
	#createHookMapRegisterTaps<H extends liteTapable.Hook<any, any, any>>(
		registerKind: binding.RegisterJsTapKind,
		getHookMap: () => liteTapable.HookMap<H>,
		createTap: (queried: liteTapable.QueriedHookMap<H>) => any
	): (stages: number[]) => binding.JsTap[] {
		const that = new WeakRef(this);
		const getTaps = (stages: number[]) => {
			const compiler = that.deref()!;
			const map = getHookMap();
			if (!map.isUsed()) return [];
			const breakpoints = [
				liteTapable.minStage,
				...stages,
				liteTapable.maxStage
			];
			const jsTaps: binding.JsTap[] = [];
			for (let i = 0; i < breakpoints.length - 1; i++) {
				const from = breakpoints[i];
				const to = breakpoints[i + 1];
				const stageRange = [from, to] as const;
				const queried = map.queryStageRange(stageRange);
				if (!queried.isUsed()) continue;
				jsTaps.push({
					function: createTap(queried),
					stage: liteTapable.safeStage(from + 1)
				});
			}
			compiler.#decorateJsTaps(jsTaps);
			return jsTaps;
		};
		getTaps.registerKind = registerKind;
		getTaps.getHookMap = getHookMap;
		return getTaps;
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 * @internal
	 */
	__internal__registerBuiltinPlugin(plugin: binding.BuiltinPlugin) {
		this.#builtinPlugins.push(plugin);
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 * @internal
	 */
	__internal__getModuleExecutionResult(id: number) {
		return this.#moduleExecutionResultsMap.get(id);
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 * @internal
	 */
	__internal__get_compilation() {
		return this.#compilation;
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 * @internal
	 */
	__internal__get_compilation_params() {
		return this.#compilationParams;
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 * @internal
	 */
	__internal__get_module_execution_results_map() {
		return this.#moduleExecutionResultsMap;
	}
}

export { Compiler };
