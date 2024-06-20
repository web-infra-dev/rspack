/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Compiler.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import * as binding from "@rspack/binding";
import * as tapable from "tapable";
import { Callback, SyncBailHook, SyncHook } from "tapable";
import type Watchpack from "watchpack";

import { Compilation, CompilationParams } from "./Compilation";
import { ContextModuleFactory } from "./ContextModuleFactory";
import { RuleSetCompiler } from "./RuleSetCompiler";
import { Stats } from "./Stats";
import {
	EntryNormalized,
	OutputNormalized,
	RspackOptionsNormalized,
	RspackPluginInstance,
	getRawOptions
} from "./config";
import { rspack } from "./index";
import * as liteTapable from "./lite-tapable";
import ResolverFactory = require("./ResolverFactory");
import { ThreadsafeWritableNodeFS } from "./FileSystem";
import ConcurrentCompilationError from "./error/ConcurrentCompilationError";
import Cache = require("./lib/Cache");
import CacheFacade = require("./lib/CacheFacade");
import { Source } from "webpack-sources";

import { Chunk } from "./Chunk";
import ExecuteModulePlugin from "./ExecuteModulePlugin";
import { FileSystemInfoEntry } from "./FileSystemInfo";
import {
	CodeGenerationResult,
	ContextModuleFactoryAfterResolveResult,
	Module,
	ResolveData
} from "./Module";
import {
	NormalModuleCreateData,
	NormalModuleFactory
} from "./NormalModuleFactory";
import {
	RuntimeGlobals,
	__from_binding_runtime_globals,
	__to_binding_runtime_globals
} from "./RuntimeGlobals";
import { Watching } from "./Watching";
import {
	JavascriptModulesPlugin,
	JsLoaderRspackPlugin
} from "./builtin-plugin";
import { canInherentFromParent } from "./builtin-plugin/base";
import { applyRspackOptionsDefaults } from "./config/defaults";
import { tryRunOrWebpackError } from "./lib/HookWebpackError";
import { Logger } from "./logging/Logger";
import { unsupported } from "./util";
import { assertNotNill } from "./util/assertNotNil";
import { checkVersion } from "./util/bindingVersionCheck";
import { createHash } from "./util/createHash";
import { OutputFileSystem, WatchFileSystem } from "./util/fs";
import { makePathsRelative } from "./util/identifier";

export interface AssetEmittedInfo {
	content: Buffer;
	source: Source;
	outputPath: string;
	targetPath: string;
	compilation: Compilation;
}

class Compiler {
	#instance?: binding.Rspack;
	#initial: boolean;

	#compilation?: Compilation;
	#compilationParams?: CompilationParams;

	#builtinPlugins: binding.BuiltinPlugin[];

	#moduleExecutionResultsMap: Map<number, any>;

	#nonSkippableRegisters: binding.RegisterJsTapKind[];
	#registers?: binding.RegisterJsTaps;

	#ruleSet: RuleSetCompiler;

	hooks: {
		done: tapable.AsyncSeriesHook<Stats>;
		afterDone: tapable.SyncHook<Stats>;
		thisCompilation: liteTapable.SyncHook<[Compilation, CompilationParams]>;
		compilation: liteTapable.SyncHook<[Compilation, CompilationParams]>;
		invalid: tapable.SyncHook<[string | null, number]>;
		compile: tapable.SyncHook<[CompilationParams]>;
		normalModuleFactory: tapable.SyncHook<NormalModuleFactory>;
		contextModuleFactory: tapable.SyncHook<ContextModuleFactory>;
		initialize: tapable.SyncHook<[]>;
		shouldEmit: liteTapable.SyncBailHook<[Compilation], boolean>;
		infrastructureLog: tapable.SyncBailHook<[string, string, any[]], true>;
		beforeRun: tapable.AsyncSeriesHook<[Compiler]>;
		run: tapable.AsyncSeriesHook<[Compiler]>;
		emit: liteTapable.AsyncSeriesHook<[Compilation]>;
		assetEmitted: liteTapable.AsyncSeriesHook<[string, AssetEmittedInfo]>;
		afterEmit: liteTapable.AsyncSeriesHook<[Compilation]>;
		failed: tapable.SyncHook<[Error]>;
		shutdown: tapable.AsyncSeriesHook<[]>;
		watchRun: tapable.AsyncSeriesHook<[Compiler]>;
		watchClose: tapable.SyncHook<[]>;
		environment: tapable.SyncHook<[]>;
		afterEnvironment: tapable.SyncHook<[]>;
		afterPlugins: tapable.SyncHook<[Compiler]>;
		afterResolvers: tapable.SyncHook<[Compiler]>;
		make: liteTapable.AsyncParallelHook<[Compilation]>;
		beforeCompile: tapable.AsyncSeriesHook<[CompilationParams]>;
		afterCompile: tapable.AsyncSeriesHook<[Compilation]>;
		finishMake: liteTapable.AsyncSeriesHook<[Compilation]>;
		entryOption: tapable.SyncBailHook<[string, EntryNormalized], any>;
	};

	webpack: typeof rspack;
	name?: string;
	parentCompilation?: Compilation;
	root: Compiler;
	outputPath: string;

	running: boolean;
	idle: boolean;
	resolverFactory: ResolverFactory;
	infrastructureLogger: any;
	watching?: Watching;

	inputFileSystem: any;
	intermediateFileSystem: any;
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
			initialize: new SyncHook([]),
			shouldEmit: new liteTapable.SyncBailHook(["compilation"]),
			done: new tapable.AsyncSeriesHook<Stats>(["stats"]),
			afterDone: new tapable.SyncHook<Stats>(["stats"]),
			beforeRun: new tapable.AsyncSeriesHook(["compiler"]),
			run: new tapable.AsyncSeriesHook(["compiler"]),
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
			invalid: new SyncHook(["filename", "changeTime"]),
			compile: new SyncHook(["params"]),
			infrastructureLog: new SyncBailHook(["origin", "type", "args"]),
			failed: new SyncHook(["error"]),
			shutdown: new tapable.AsyncSeriesHook([]),
			normalModuleFactory: new tapable.SyncHook<NormalModuleFactory>([
				"normalModuleFactory"
			]),
			contextModuleFactory: new tapable.SyncHook<ContextModuleFactory>([
				"contextModuleFactory"
			]),
			watchRun: new tapable.AsyncSeriesHook(["compiler"]),
			watchClose: new tapable.SyncHook([]),
			environment: new tapable.SyncHook([]),
			afterEnvironment: new tapable.SyncHook([]),
			afterPlugins: new tapable.SyncHook(["compiler"]),
			afterResolvers: new tapable.SyncHook(["compiler"]),
			make: new liteTapable.AsyncParallelHook(["compilation"]),
			beforeCompile: new tapable.AsyncSeriesHook(["params"]),
			afterCompile: new tapable.AsyncSeriesHook(["compilation"]),
			finishMake: new liteTapable.AsyncSeriesHook(["compilation"]),
			entryOption: new tapable.SyncBailHook(["context", "entry"])
		};

		this.webpack = rspack;
		this.root = this;
		this.outputPath = "";

		this.inputFileSystem = null;
		this.intermediateFileSystem = null;
		this.outputFileSystem = null;
		this.watchFileSystem = null;

		this.records = {};

		this.resolverFactory = new ResolverFactory();
		this.options = options;
		this.context = context;
		this.cache = new Cache();

		this.compilerPath = "";

		this.running = false;

		this.idle = false;

		this.watchMode = false;

		new JsLoaderRspackPlugin(this).apply(this);
		new ExecuteModulePlugin().apply(this);
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
			this.options.output.hashFunction
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
		return new Logger(
			(type, args) => {
				if (typeof name === "function") {
					name = name();
					if (!name) {
						throw new TypeError(
							"Compiler.getInfrastructureLogger(name) called with a function not returning a name"
						);
					}
				} else {
					if (
						this.hooks.infrastructureLog.call(name, type, args) === undefined
					) {
						if (this.infrastructureLogger !== undefined) {
							this.infrastructureLogger(name, type, args);
						}
					}
				}
			},
			(childName): any => {
				if (typeof name === "function") {
					if (typeof childName === "function") {
						// @ts-expect-error
						return this.getInfrastructureLogger(_ => {
							if (typeof name === "function") {
								name = name();
								if (!name) {
									throw new TypeError(
										"Compiler.getInfrastructureLogger(name) called with a function not returning a name"
									);
								}
							}
							if (typeof childName === "function") {
								childName = childName();
								if (!childName) {
									throw new TypeError(
										"Logger.getChildLogger(name) called with a function not returning a name"
									);
								}
							}
							return `${name}/${childName}`;
						});
					} else {
						return this.getInfrastructureLogger(() => {
							if (typeof name === "function") {
								name = name();
								if (!name) {
									throw new TypeError(
										"Compiler.getInfrastructureLogger(name) called with a function not returning a name"
									);
								}
							}
							return `${name}/${childName}`;
						});
					}
				} else {
					if (typeof childName === "function") {
						return this.getInfrastructureLogger(() => {
							if (typeof childName === "function") {
								childName = childName();
								if (!childName) {
									throw new TypeError(
										"Logger.getChildLogger(name) called with a function not returning a name"
									);
								}
							}
							return `${name}/${childName}`;
						});
					} else {
						return this.getInfrastructureLogger(`${name}/${childName}`);
					}
				}
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
		handler: Callback<Error, Stats>
	): Watching {
		if (this.running) {
			// @ts-expect-error
			return handler(new ConcurrentCompilationError());
		}
		this.running = true;
		this.watchMode = true;
		// @ts-expect-error
		this.watching = new Watching(this, watchOptions, handler);
		return this.watching;
	}

	/**
	 * @param callback - signals when the call finishes
	 */
	run(callback: Callback<Error, Stats>) {
		if (this.running) {
			return callback(new ConcurrentCompilationError());
		}
		const startTime = Date.now();
		this.running = true;
		const doRun = () => {
			// @ts-expect-error
			const finalCallback = (err, stats?) => {
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
				this.hooks.afterDone.call(stats);
			};
			this.hooks.beforeRun.callAsync(this, err => {
				if (err) {
					return finalCallback(err);
				}
				this.hooks.run.callAsync(this, err => {
					if (err) {
						return finalCallback(err);
					}

					this.compile(err => {
						if (err) {
							return finalCallback(err);
						}
						this.#compilation!.startTime = startTime;
						this.#compilation!.endTime = Date.now();
						const stats = new Stats(this.#compilation!);
						this.hooks.done.callAsync(stats, err => {
							if (err) {
								return finalCallback(err);
							} else {
								return finalCallback(null, stats);
							}
						});
					});
				});
			});
		};

		if (this.idle) {
			this.cache.endIdle(err => {
				if (err) return callback(err);

				this.idle = false;
				doRun();
			});
		} else {
			doRun();
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
		if (this.inputFileSystem && this.inputFileSystem.purge) {
			this.inputFileSystem.purge();
		}
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
		childCompiler.resolverFactory = this.resolverFactory;
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

		for (const name in this.hooks) {
			if (canInherentFromParent(name as keyof Compiler["hooks"])) {
				//@ts-ignore
				if (childCompiler.hooks[name]) {
					//@ts-ignore
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

	compile(callback: Callback<Error, Compilation>) {
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

	#build(callback?: (error: Error | null) => void) {
		this.#getInstance((error, instance) => {
			if (error) {
				return callback?.(error);
			}
			if (!this.#initial) {
				instance!.rebuild(
					Array.from(this.modifiedFiles || []),
					Array.from(this.removedFiles || []),
					error => {
						if (error) {
							return callback?.(error);
						}
						callback?.(null);
					}
				);
				return;
			}
			this.#initial = false;
			instance!.build(error => {
				if (error) {
					return callback?.(error);
				}
				callback?.(null);
			});
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

	#createCompilation(native: binding.JsCompilation): Compilation {
		const compilation = new Compilation(this, native);
		compilation.name = this.name;
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
		const normalModuleFactory = new NormalModuleFactory();
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
		callback: (error: Error | null, instance?: binding.Rspack) => void
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

		this.#registers = {
			registerCompilerThisCompilationTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilerThisCompilation,
				() => this.hooks.thisCompilation,
				queried => (native: binding.JsCompilation) => {
					if (this.#compilation === undefined) {
						this.#createCompilation(native);
					}
					queried.call(this.#compilation!, this.#compilationParams!);
				}
			),
			registerCompilerCompilationTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilerCompilation,
				() => this.hooks.compilation,
				queried => () =>
					queried.call(this.#compilation!, this.#compilationParams!)
			),
			registerCompilerMakeTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilerMake,
				() => this.hooks.make,
				queried => async () => await queried.promise(this.#compilation!)
			),
			registerCompilerFinishMakeTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilerFinishMake,
				() => this.hooks.finishMake,
				queried => async () => await queried.promise(this.#compilation!)
			),
			registerCompilerShouldEmitTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilerShouldEmit,
				() => this.hooks.shouldEmit,
				queried => () => queried.call(this.#compilation!)
			),
			registerCompilerEmitTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilerEmit,
				() => this.hooks.emit,
				queried => async () => await queried.promise(this.#compilation!)
			),
			registerCompilerAfterEmitTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilerAfterEmit,
				() => this.hooks.afterEmit,
				queried => async () => await queried.promise(this.#compilation!)
			),
			registerCompilerAssetEmittedTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilerAssetEmitted,
				() => this.hooks.assetEmitted,
				queried =>
					async ({
						filename,
						targetPath,
						outputPath
					}: binding.JsAssetEmittedArgs) => {
						return queried.promise(filename, {
							compilation: this.#compilation!,
							targetPath,
							outputPath,
							get source() {
								return this.compilation!.getAsset(filename)?.source;
							},
							get content() {
								return this.source?.buffer();
							}
						});
					}
			),
			registerCompilationAdditionalTreeRuntimeRequirements:
				this.#createHookRegisterTaps(
					binding.RegisterJsTapKind
						.CompilationAdditionalTreeRuntimeRequirements,
					() => this.#compilation!.hooks.additionalTreeRuntimeRequirements,
					queried =>
						({
							chunk,
							runtimeRequirements
						}: binding.JsAdditionalTreeRuntimeRequirementsArg) => {
							const set = __from_binding_runtime_globals(runtimeRequirements);
							queried.call(
								Chunk.__from_binding(chunk, this.#compilation!),
								set
							);
							return {
								runtimeRequirements: __to_binding_runtime_globals(set)
							};
						}
				),
			registerCompilationRuntimeModuleTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationRuntimeModule,
				() => this.#compilation!.hooks.runtimeModule,
				queried =>
					({ module, chunk }: binding.JsRuntimeModuleArg) => {
						const originSource = module.source?.source;
						queried.call(
							module,
							Chunk.__from_binding(chunk, this.#compilation!)
						);
						const newSource = module.source?.source;
						if (newSource && newSource !== originSource) {
							return module;
						}
						return;
					}
			),
			registerCompilationBuildModuleTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationBuildModule,
				() => this.#compilation!.hooks.buildModule,
				queired => (m: binding.JsModule) =>
					queired.call(Module.__from_binding(m, this.#compilation))
			),
			registerCompilationStillValidModuleTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationStillValidModule,
				() => this.#compilation!.hooks.stillValidModule,
				queired => (m: binding.JsModule) =>
					queired.call(Module.__from_binding(m, this.#compilation))
			),
			registerCompilationSucceedModuleTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationSucceedModule,
				() => this.#compilation!.hooks.succeedModule,
				queired => (m: binding.JsModule) =>
					queired.call(Module.__from_binding(m, this.#compilation))
			),
			registerCompilationExecuteModuleTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationExecuteModule,
				() => this.#compilation!.hooks.executeModule,
				queried =>
					({
						entry,
						id,
						codegenResults,
						runtimeModules
					}: binding.JsExecuteModuleArg) => {
						const __webpack_require__: any = (id: string) => {
							const cached = moduleCache[id];
							if (cached !== undefined) {
								if (cached.error) throw cached.error;
								return cached.exports;
							}

							var execOptions = {
								id,
								module: {
									id,
									exports: {},
									loaded: false,
									error: undefined
								},
								require: __webpack_require__
							};

							interceptModuleExecution.forEach(
								(handler: (execOptions: any) => void) => handler(execOptions)
							);

							const result = codegenResults.map[id]["build time"];
							const moduleObject = execOptions.module;

							if (id) moduleCache[id] = moduleObject;

							tryRunOrWebpackError(
								() =>
									queried.call(
										{
											codeGenerationResult: new CodeGenerationResult(result),
											moduleObject
										},
										{ __webpack_require__ }
									),
								"Compilation.hooks.executeModule"
							);
							moduleObject.loaded = true;
							return moduleObject.exports;
						};

						const moduleCache: Record<string, any> = (__webpack_require__[
							RuntimeGlobals.moduleCache.replace(
								`${RuntimeGlobals.require}.`,
								""
							)
						] = {});
						const interceptModuleExecution = (__webpack_require__[
							RuntimeGlobals.interceptModuleExecution.replace(
								`${RuntimeGlobals.require}.`,
								""
							)
						] = []);

						for (const runtimeModule of runtimeModules) {
							__webpack_require__(runtimeModule);
						}

						const executeResult = __webpack_require__(entry);

						this.#moduleExecutionResultsMap.set(id, executeResult);
					}
			),
			registerCompilationFinishModulesTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationFinishModules,
				() => this.#compilation!.hooks.finishModules,
				queried => async () => await queried.promise(this.#compilation!.modules)
			),
			registerCompilationOptimizeModulesTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationOptimizeModules,
				() => this.#compilation!.hooks.optimizeModules,
				queried => () => queried.call(this.#compilation!.modules.values())
			),
			registerCompilationAfterOptimizeModulesTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationAfterOptimizeModules,
				() => this.#compilation!.hooks.afterOptimizeModules,
				queried => () => {
					queried.call(this.#compilation!.modules.values());
				}
			),
			registerCompilationOptimizeTreeTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationOptimizeTree,
				() => this.#compilation!.hooks.optimizeTree,
				queried => async () =>
					await queried.promise(
						this.#compilation!.chunks,
						this.#compilation!.modules
					)
			),
			registerCompilationOptimizeChunkModulesTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationOptimizeChunkModules,
				() => this.#compilation!.hooks.optimizeChunkModules,
				queried => async () =>
					await queried.promise(
						this.#compilation!.chunks,
						this.#compilation!.modules
					)
			),
			registerCompilationChunkHashTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationChunkHash,
				() => this.#compilation!.hooks.chunkHash,
				queried => (chunk: binding.JsChunk) => {
					const hash = createHash(this.options.output.hashFunction);
					queried.call(Chunk.__from_binding(chunk, this.#compilation!), hash);
					const digestResult = hash.digest(this.options.output.hashDigest);
					return Buffer.from(digestResult);
				}
			),
			registerCompilationChunkAssetTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationChunkAsset,
				() => this.#compilation!.hooks.chunkAsset,
				queried =>
					({ chunk, filename }: binding.JsChunkAssetArgs) =>
						queried.call(
							Chunk.__from_binding(chunk, this.#compilation!),
							filename
						)
			),
			registerCompilationProcessAssetsTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationProcessAssets,
				() => this.#compilation!.hooks.processAssets,
				queried => async () => await queried.promise(this.#compilation!.assets)
			),
			registerCompilationAfterProcessAssetsTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationAfterProcessAssets,
				() => this.#compilation!.hooks.afterProcessAssets,
				queried => () => queried.call(this.#compilation!.assets)
			),
			registerCompilationAfterSealTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.CompilationAfterSeal,
				() => this.#compilation!.hooks.afterSeal,
				queried => async () => await queried.promise()
			),
			registerNormalModuleFactoryBeforeResolveTaps:
				this.#createHookRegisterTaps(
					binding.RegisterJsTapKind.NormalModuleFactoryBeforeResolve,
					() =>
						this.#compilationParams!.normalModuleFactory.hooks.beforeResolve,
					queried => async (resolveData: binding.JsBeforeResolveArgs) => {
						const normalizedResolveData: ResolveData = {
							request: resolveData.request,
							context: resolveData.context,
							fileDependencies: [],
							missingDependencies: [],
							contextDependencies: []
						};
						const ret = await queried.promise(normalizedResolveData);
						resolveData.request = normalizedResolveData.request;
						resolveData.context = normalizedResolveData.context;
						return [ret, resolveData];
					}
				),
			registerNormalModuleFactoryResolveForSchemeTaps:
				this.#createHookMapRegisterTaps(
					binding.RegisterJsTapKind.NormalModuleFactoryResolveForScheme,
					() =>
						this.#compilationParams!.normalModuleFactory.hooks.resolveForScheme,
					queried => async (args: binding.JsResolveForSchemeArgs) => {
						const ret = await queried
							.for(args.scheme)
							.promise(args.resourceData);
						return [ret, args.resourceData];
					}
				),
			registerNormalModuleFactoryAfterResolveTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.NormalModuleFactoryAfterResolve,
				() => this.#compilationParams!.normalModuleFactory.hooks.afterResolve,
				queried => async (arg: binding.JsAfterResolveData) => {
					const data: ResolveData = {
						request: arg.request,
						context: arg.context,
						fileDependencies: arg.fileDependencies,
						missingDependencies: arg.missingDependencies,
						contextDependencies: arg.contextDependencies,
						createData: arg.createData
					};
					const ret = await queried.promise(data);
					return [ret, data.createData];
				}
			),
			registerNormalModuleFactoryCreateModuleTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.NormalModuleFactoryCreateModule,
				() => this.#compilationParams!.normalModuleFactory.hooks.createModule,
				queried =>
					async (args: binding.JsNormalModuleFactoryCreateModuleArgs) => {
						const data: NormalModuleCreateData = {
							...args,
							settings: {}
						};
						await queried.promise(data, {});
					}
			),
			registerContextModuleFactoryBeforeResolveTaps:
				this.#createHookRegisterTaps(
					binding.RegisterJsTapKind.ContextModuleFactoryBeforeResolve,
					() =>
						this.#compilationParams!.contextModuleFactory.hooks.beforeResolve,
					queried =>
						async (
							bindingData:
								| false
								| binding.JsContextModuleFactoryBeforeResolveData
						) => {
							return queried.promise(bindingData);
						}
				),
			registerContextModuleFactoryAfterResolveTaps:
				this.#createHookRegisterTaps(
					binding.RegisterJsTapKind.ContextModuleFactoryAfterResolve,
					() =>
						this.#compilationParams!.contextModuleFactory.hooks.afterResolve,
					queried =>
						async (
							bindingData:
								| false
								| binding.JsContextModuleFactoryAfterResolveData
						) => {
							const data = bindingData
								? ({
										resource: bindingData.resource,
										regExp: bindingData.regExp
											? new RegExp(
													bindingData.regExp.source,
													bindingData.regExp.flags
												)
											: undefined,
										request: bindingData.request,
										context: bindingData.context,
										// TODO: Dependencies are not fully supported yet; this is a placeholder to prevent errors in moment-locales-webpack-plugin.
										dependencies: []
									} satisfies ContextModuleFactoryAfterResolveResult)
								: false;
							const ret = await queried.promise(data);
							const result = ret
								? ({
										resource: ret.resource,
										context: ret.context,
										request: ret.request,
										regExp: ret.regExp
											? {
													source: ret.regExp.source,
													flags: ret.regExp.flags
												}
											: undefined
									} satisfies binding.JsContextModuleFactoryAfterResolveData)
								: false;
							return result;
						}
				),
			registerJavascriptModulesChunkHashTaps: this.#createHookRegisterTaps(
				binding.RegisterJsTapKind.JavascriptModulesChunkHash,
				() =>
					JavascriptModulesPlugin.getCompilationHooks(this.#compilation!)
						.chunkHash,
				queried => (chunk: binding.JsChunk) => {
					const hash = createHash(this.options.output.hashFunction);
					queried.call(Chunk.__from_binding(chunk, this.#compilation!), hash);
					const digestResult = hash.digest(this.options.output.hashDigest);
					return Buffer.from(digestResult);
				}
			)
		};

		this.#instance = new instanceBinding.Rspack(
			rawOptions,
			this.#builtinPlugins,
			this.#registers,
			ThreadsafeWritableNodeFS.__to_binding(this.outputFileSystem!)
		);

		callback(null, this.#instance);
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
			this.#getInstance((error, instance) => {
				instance!.setNonSkippableRegisters(kinds);
				this.#nonSkippableRegisters = kinds;
			});
		}
	}

	#decorateJsTaps(jsTaps: binding.JsTap[]) {
		if (jsTaps.length > 0) {
			const last = jsTaps[jsTaps.length - 1];
			const old = last.function;
			last.function = (...args) => {
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

	#createHookRegisterTaps<T, R, A>(
		registerKind: binding.RegisterJsTapKind,
		getHook: () => liteTapable.Hook<T, R, A>,
		createTap: (queried: liteTapable.QueriedHook<T, R, A>) => any
	): (stages: number[]) => binding.JsTap[] {
		const getTaps = (stages: number[]) => {
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
			this.#decorateJsTaps(jsTaps);
			return jsTaps;
		};
		getTaps.registerKind = registerKind;
		getTaps.getHook = getHook;
		return getTaps;
	}

	#createHookMapRegisterTaps<H extends liteTapable.Hook<any, any, any>>(
		registerKind: binding.RegisterJsTapKind,
		getHookMap: () => liteTapable.HookMap<H>,
		createTap: (queried: liteTapable.QueriedHookMap<H>) => any
	): (stages: number[]) => binding.JsTap[] {
		const getTaps = (stages: number[]) => {
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
			this.#decorateJsTaps(jsTaps);
			return jsTaps;
		};
		getTaps.registerKind = registerKind;
		getTaps.getHookMap = getHookMap;
		return getTaps;
	}

	__internal__registerBuiltinPlugin(plugin: binding.BuiltinPlugin) {
		this.#builtinPlugins.push(plugin);
	}

	__internal__getModuleExecutionResult(id: number) {
		return this.#moduleExecutionResultsMap.get(id);
	}
}

export { Compiler };
