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
import { rspack } from "./index";
import fs from "fs";
import * as tapable from "tapable";
import * as liteTapable from "./lite-tapable";
import { Callback, SyncBailHook, SyncHook } from "tapable";
import type { WatchOptions } from "watchpack";
import {
	getRawOptions,
	EntryNormalized,
	OutputNormalized,
	RspackOptionsNormalized,
	RspackPluginInstance
} from "./config";
import { RuleSetCompiler } from "./RuleSetCompiler";
import { Stats } from "./Stats";
import { Compilation, CompilationParams } from "./Compilation";
import { ContextModuleFactory } from "./ContextModuleFactory";
import ResolverFactory from "./ResolverFactory";
import ConcurrentCompilationError from "./error/ConcurrentCompilationError";
import { createThreadsafeNodeFSFromRaw } from "./fileSystem";
import Cache from "./lib/Cache";
import CacheFacade from "./lib/CacheFacade";
import { Logger } from "./logging/Logger";
import { NormalModuleFactory } from "./NormalModuleFactory";
import { WatchFileSystem } from "./util/fs";
import { checkVersion } from "./util/bindingVersionCheck";
import { Watching } from "./Watching";
import { NormalModule } from "./NormalModule";
import { deprecated_resolveBuiltins } from "./builtin-plugin";
import { applyEntryOptions } from "./rspackOptionsApply";
import { applyRspackOptionsDefaults } from "./config/defaults";
import { assertNotNill } from "./util/assertNotNil";
import { FileSystemInfoEntry } from "./FileSystemInfo";
import { RuntimeGlobals } from "./RuntimeGlobals";
import { tryRunOrWebpackError } from "./lib/HookWebpackError";
import { CodeGenerationResult, Module } from "./Module";
import { canInherentFromParent } from "./builtin-plugin/base";
import ExecuteModulePlugin from "./ExecuteModulePlugin";
import { Chunk } from "./Chunk";

class Compiler {
	#instance?: binding.Rspack;

	webpack = rspack;
	compilation?: Compilation;
	compilationParams?: CompilationParams;
	// TODO: remove this after remove rebuild on the rust side.
	first: boolean = true;
	builtinPlugins: binding.BuiltinPlugin[];
	root: Compiler;
	running: boolean;
	idle: boolean;
	resolverFactory: ResolverFactory;
	infrastructureLogger: any;
	watching?: Watching;
	outputPath!: string;
	name?: string;
	inputFileSystem: any;
	outputFileSystem: typeof import("fs");
	ruleSet: RuleSetCompiler;
	// @ts-expect-error
	watchFileSystem: WatchFileSystem;
	intermediateFileSystem: any;
	// @ts-expect-error
	watchMode: boolean;
	context: string;
	cache: Cache;
	compilerPath: string;
	modifiedFiles?: ReadonlySet<string>;
	removedFiles?: ReadonlySet<string>;
	fileTimestamps?: ReadonlyMap<string, FileSystemInfoEntry | "ignore" | null>;
	contextTimestamps?: ReadonlyMap<
		string,
		FileSystemInfoEntry | "ignore" | null
	>;
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
		emit: tapable.AsyncSeriesHook<[Compilation]>;
		assetEmitted: tapable.AsyncSeriesHook<[string, any]>;
		afterEmit: tapable.AsyncSeriesHook<[Compilation]>;
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
	options: RspackOptionsNormalized;
	#disabledHooks: string[];
	parentCompilation?: Compilation;

	#moduleExecutionResultsMap: Map<number, any>;

	constructor(context: string, options: RspackOptionsNormalized) {
		this.outputFileSystem = fs;
		this.options = options;
		this.cache = new Cache();
		this.compilerPath = "";
		this.builtinPlugins = [];
		this.root = this;
		this.ruleSet = new RuleSetCompiler();
		this.running = false;
		this.idle = false;
		this.context = context;
		this.resolverFactory = new ResolverFactory();
		this.modifiedFiles = undefined;
		this.removedFiles = undefined;
		this.hooks = {
			initialize: new SyncHook([]),
			shouldEmit: new liteTapable.SyncBailHook(["compilation"]),
			done: new tapable.AsyncSeriesHook<Stats>(["stats"]),
			afterDone: new tapable.SyncHook<Stats>(["stats"]),
			beforeRun: new tapable.AsyncSeriesHook(["compiler"]),
			run: new tapable.AsyncSeriesHook(["compiler"]),
			emit: new tapable.AsyncSeriesHook(["compilation"]),
			assetEmitted: new tapable.AsyncSeriesHook(["file", "info"]),
			afterEmit: new tapable.AsyncSeriesHook(["compilation"]),
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
		this.modifiedFiles = undefined;
		this.removedFiles = undefined;
		this.#disabledHooks = [];
		this.#moduleExecutionResultsMap = new Map();

		new ExecuteModulePlugin().apply(this);
	}

	/**
	 * @param {string} name cache name
	 * @returns {CacheFacade} the cache facade instance
	 */
	getCache(name: string): CacheFacade {
		return new CacheFacade(
			this.cache,
			`${this.compilerPath}${name}`,
			this.options.output.hashFunction
		);
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
		// TODO: remove this in v0.6
		if (!options.experiments.rspackFuture!.disableApplyEntryLazily) {
			applyEntryOptions(this, options);
		}
		// TODO: remove this when drop support for builtins options
		options.builtins = deprecated_resolveBuiltins(
			options.builtins,
			options
		) as any;
		const rawOptions = getRawOptions(options, this);

		const instanceBinding: typeof binding = require("@rspack/binding");

		this.#instance = new instanceBinding.Rspack(
			rawOptions,
			this.builtinPlugins,
			{
				emit: this.#emit.bind(this),
				assetEmitted: this.#assetEmitted.bind(this),
				afterEmit: this.#afterEmit.bind(this),
				afterProcessAssets: this.#afterProcessAssets.bind(this),
				optimizeModules: this.#optimizeModules.bind(this),
				afterOptimizeModules: this.#afterOptimizeModules.bind(this),
				optimizeTree: this.#optimizeTree.bind(this),
				optimizeChunkModules: this.#optimizeChunkModules.bind(this),
				normalModuleFactoryCreateModule:
					this.#normalModuleFactoryCreateModule.bind(this),
				normalModuleFactoryResolveForScheme:
					this.#normalModuleFactoryResolveForScheme.bind(this),
				afterResolve: this.#afterResolve.bind(this),
				contextModuleFactoryBeforeResolve:
					this.#contextModuleFactoryBeforeResolve.bind(this),
				contextModuleFactoryAfterResolve:
					this.#contextModuleFactoryAfterResolve.bind(this)
			},
			{
				registerCompilerThisCompilationTaps: this.#createRegisterTaps(
					() => this.hooks.thisCompilation,
					queried => (native: binding.JsCompilation) => {
						if (this.compilation === undefined) {
							this.#createCompilation(native);
						}
						queried.call(this.compilation!, this.compilationParams!);
					}
				),
				registerCompilerCompilationTaps: this.#createRegisterTaps(
					() => this.hooks.compilation,
					queried => () =>
						queried.call(this.compilation!, this.compilationParams!)
				),
				registerCompilerMakeTaps: this.#createRegisterTaps(
					() => this.hooks.make,
					queried => async () => await queried.promise(this.compilation!)
				),
				registerCompilerFinishMakeTaps: this.#createRegisterTaps(
					() => this.hooks.finishMake,
					queried => async () => await queried.promise(this.compilation!)
				),
				registerCompilerShouldEmitTaps: this.#createRegisterTaps(
					() => this.hooks.shouldEmit,
					queried => () => queried.call(this.compilation!)
				),
				registerCompilationRuntimeModuleTaps: this.#createRegisterTaps(
					() => this.compilation!.hooks.runtimeModule,
					queried =>
						({ module, chunk }: binding.JsRuntimeModuleArg) => {
							const originSource = module.source?.source;
							queried.call(
								module,
								Chunk.__from_binding(chunk, this.compilation!)
							);
							const newSource = module.source?.source;
							if (newSource && newSource !== originSource) {
								return module;
							}
							return;
						}
				),
				registerCompilationBuildModuleTaps: this.#createRegisterTaps(
					() => this.compilation!.hooks.buildModule,
					queired => (m: binding.JsModule) =>
						queired.call(Module.__from_binding(m))
				),
				registerCompilationStillValidModuleTaps: this.#createRegisterTaps(
					() => this.compilation!.hooks.stillValidModule,
					queired => (m: binding.JsModule) =>
						queired.call(Module.__from_binding(m))
				),
				registerCompilationSucceedModuleTaps: this.#createRegisterTaps(
					() => this.compilation!.hooks.succeedModule,
					queired => (m: binding.JsModule) =>
						queired.call(Module.__from_binding(m))
				),
				registerCompilationExecuteModuleTaps: this.#createRegisterTaps(
					() => this.compilation!.hooks.executeModule,
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
				registerCompilationFinishModulesTaps: this.#createRegisterTaps(
					() => this.compilation!.hooks.finishModules,
					queried => async () =>
						await queried.promise(this.compilation!.modules)
				),
				registerCompilationChunkAssetTaps: this.#createRegisterTaps(
					() => this.compilation!.hooks.chunkAsset,
					queried =>
						({ chunk, filename }: binding.JsChunkAssetArgs) =>
							queried.call(
								Chunk.__from_binding(chunk, this.compilation!),
								filename
							)
				),
				registerCompilationProcessAssetsTaps: this.#createRegisterTaps(
					() => this.compilation!.hooks.processAssets,
					queried => async () => await queried.promise(this.compilation!.assets)
				),
				registerNormalModuleFactoryBeforeResolveTaps: this.#createRegisterTaps(
					() => this.compilationParams!.normalModuleFactory.hooks.beforeResolve,
					queried => async (resolveData: binding.JsBeforeResolveArgs) => {
						const normalizedResolveData = {
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
				)
			},
			createThreadsafeNodeFSFromRaw(this.outputFileSystem)
		);

		callback(null, this.#instance);
	}

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
			},
			// TODO: check why we need to have builtins otherwise this.#instance will fail to initialize Rspack
			builtins: this.options.builtins
		};
		applyRspackOptionsDefaults(options);
		const childCompiler = new Compiler(this.context, options);
		childCompiler.name = compilerName;
		childCompiler.outputPath = this.outputPath;
		childCompiler.inputFileSystem = this.inputFileSystem;
		// childCompiler.outputFileSystem = null;
		childCompiler.resolverFactory = this.resolverFactory;
		childCompiler.modifiedFiles = this.modifiedFiles;
		childCompiler.removedFiles = this.removedFiles;
		// childCompiler.fileTimestamps = this.fileTimestamps;
		// childCompiler.contextTimestamps = this.contextTimestamps;
		// childCompiler.fsStartTime = this.fsStartTime;
		childCompiler.cache = this.cache;
		childCompiler.compilerPath = `${this.compilerPath}${compilerName}|${compilerIndex}|`;
		// childCompiler._backCompat = this._backCompat;

		// const relativeCompilerName = makePathsRelative(
		// 	this.context,
		// 	compilerName,
		// 	this.root
		// );
		// if (!this.records[relativeCompilerName]) {
		// 	this.records[relativeCompilerName] = [];
		// }
		// if (this.records[relativeCompilerName][compilerIndex]) {
		// 	childCompiler.records = this.records[relativeCompilerName][compilerIndex];
		// } else {
		// 	this.records[relativeCompilerName].push((childCompiler.records = {}));
		// }

		childCompiler.parentCompilation = compilation;
		childCompiler.root = this.root;
		if (Array.isArray(plugins)) {
			for (const plugin of plugins) {
				if (plugin) {
					plugin.apply(childCompiler);
				}
			}
		}

		childCompiler.builtinPlugins = [
			...childCompiler.builtinPlugins,
			...this.builtinPlugins.filter(
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

	runAsChild(callback: any) {
		const finalCallback = (
			err: Error | null,
			entries?: any,
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
				entries.push(...ep.getFiles());
			}

			return finalCallback(null, entries, compilation);
		});
	}

	isChild(): boolean {
		const isRoot = this.root === this;
		return !isRoot;
	}

	getInfrastructureLogger(name: string | Function) {
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
						// @ts-expect-error
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

	#updateDisabledHooks(callback?: (error?: Error) => void) {
		const disabledHooks: string[] = [];
		type HookMap = Record<keyof binding.JsHooks, any>;
		const hookMap: HookMap = {
			emit: this.hooks.emit,
			assetEmitted: this.hooks.assetEmitted,
			afterEmit: this.hooks.afterEmit,
			afterProcessAssets: this.compilation!.hooks.afterProcessAssets,
			optimizeTree: this.compilation!.hooks.optimizeTree,
			optimizeModules: this.compilation!.hooks.optimizeModules,
			afterOptimizeModules: this.compilation!.hooks.afterOptimizeModules,
			afterResolve:
				this.compilationParams?.normalModuleFactory.hooks.afterResolve,
			optimizeChunkModules: this.compilation!.hooks.optimizeChunkModules,
			contextModuleFactoryBeforeResolve:
				this.compilationParams?.contextModuleFactory.hooks.beforeResolve,
			contextModuleFactoryAfterResolve:
				this.compilationParams?.contextModuleFactory.hooks.afterResolve,
			normalModuleFactoryCreateModule:
				this.compilationParams?.normalModuleFactory.hooks.createModule,
			normalModuleFactoryResolveForScheme:
				this.compilationParams?.normalModuleFactory.hooks.resolveForScheme
		};
		for (const [name, hook] of Object.entries(hookMap)) {
			if (
				typeof hook !== "undefined" &&
				(hook.taps
					? !hook.isUsed()
					: hook._map
						? /* hook map */ hook._map.size === 0
						: false)
			) {
				disabledHooks.push(name);
			}
		}

		// disabledHooks is in order
		if (this.#disabledHooks.join() !== disabledHooks.join()) {
			this.#getInstance((error, instance) => {
				if (error) {
					return callback?.(error);
				}
				instance!.setDisabledHooks(disabledHooks);
				this.#disabledHooks = disabledHooks;
			});
		}
	}

	async #afterProcessAssets() {
		await this.compilation!.hooks.afterProcessAssets.promise(
			this.compilation!.assets
		);
		this.#updateDisabledHooks();
	}

	async #afterResolve(resolveData: binding.AfterResolveData) {
		let res =
			await this.compilationParams!.normalModuleFactory.hooks.afterResolve.promise(
				resolveData
			);

		NormalModule.getCompilationHooks(this.compilation!).loader.tap(
			"sideEffectFreePropPlugin",
			(loaderContext: any) => {
				loaderContext._module = {
					factoryMeta: {
						sideEffectFree: !!resolveData.factoryMeta.sideEffectFree
					}
				};
			}
		);
		this.#updateDisabledHooks();
		return [res, resolveData.createData];
	}

	async #contextModuleFactoryBeforeResolve(
		resourceData: binding.JsBeforeResolveArgs
	) {
		let res =
			await this.compilationParams!.contextModuleFactory.hooks.beforeResolve.promise(
				resourceData
			);

		this.#updateDisabledHooks();
		return res;
	}

	async #contextModuleFactoryAfterResolve(
		resourceData: binding.AfterResolveData
	) {
		let res =
			await this.compilationParams!.contextModuleFactory.hooks.afterResolve.promise(
				resourceData
			);

		this.#updateDisabledHooks();
		return res;
	}

	async #normalModuleFactoryCreateModule(createData: binding.CreateModuleData) {
		const data = Object.assign({}, createData, {
			settings: {},
			matchResource: createData.resourceResolveData.resource
		});
		const nmfHooks = this.compilationParams!.normalModuleFactory.hooks;
		await nmfHooks?.createModule.promise(data, {});
		this.#updateDisabledHooks();
	}

	async #normalModuleFactoryResolveForScheme(
		input: binding.JsResolveForSchemeInput
	): Promise<binding.JsResolveForSchemeResult> {
		let stop =
			await this.compilationParams!.normalModuleFactory.hooks.resolveForScheme.for(
				input.scheme
			).promise(input.resourceData);
		this.#updateDisabledHooks();
		return {
			resourceData: input.resourceData,
			stop: stop === true
		};
	}

	async #optimizeChunkModules() {
		await this.compilation!.hooks.optimizeChunkModules.promise(
			this.compilation!.chunks,
			this.compilation!.modules
		);
		this.#updateDisabledHooks();
	}

	async #optimizeTree() {
		await this.compilation!.hooks.optimizeTree.promise(
			this.compilation!.chunks,
			this.compilation!.modules
		);
		this.#updateDisabledHooks();
	}

	async #optimizeModules() {
		await this.compilation!.hooks.optimizeModules.promise(
			this.compilation!.modules
		);
		this.#updateDisabledHooks();
	}

	async #afterOptimizeModules() {
		await this.compilation!.hooks.afterOptimizeModules.promise(
			this.compilation!.modules
		);
		this.#updateDisabledHooks();
	}

	async #emit() {
		await this.hooks.emit.promise(this.compilation!);
		this.#updateDisabledHooks();
	}
	async #assetEmitted(args: binding.JsAssetEmittedArgs) {
		const filename = args.filename;
		const info = {
			compilation: this.compilation,
			outputPath: args.outputPath,
			targetPath: args.targetPath,
			get source() {
				return this.compilation!.getAsset(args.filename)?.source;
			},
			get content() {
				return this.source?.buffer();
			}
		};
		await this.hooks.assetEmitted.promise(filename, info);
		this.#updateDisabledHooks();
	}

	async #afterEmit() {
		await this.hooks.afterEmit.promise(this.compilation!);
		this.#updateDisabledHooks();
	}

	#decorateUpdateDisabledHooks(jsTaps: binding.JsTap[]) {
		if (jsTaps.length > 0) {
			const last = jsTaps[jsTaps.length - 1];
			const old = last.function;
			last.function = (...args) => {
				const result = old(...args);
				if (result && typeof result.then === "function") {
					return result.then((r: any) => {
						this.#updateDisabledHooks();
						return r;
					});
				}
				this.#updateDisabledHooks();
				return result;
			};
		}
	}

	#createRegisterTaps<T, R, A>(
		getHook: () => liteTapable.Hook<T, R, A>,
		createTap: (queried: liteTapable.QueriedHook<T, R, A>) => any
	): (stages: number[]) => binding.JsTap[] {
		return stages => {
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
			this.#decorateUpdateDisabledHooks(jsTaps);
			return jsTaps;
		};
	}

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
						this.compilation!.startTime = startTime;
						this.compilation!.endTime = Date.now();
						const stats = new Stats(this.compilation!);
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
	/**
	 * Safety: This method is only valid to call if the previous rebuild task is finished, or there will be data races.
	 */
	build(callback?: (error: Error | null) => void) {
		this.#getInstance((error, instance) => {
			if (error) {
				return callback?.(error);
			}
			if (!this.first) {
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
			this.first = false;
			instance!.build(error => {
				if (error) {
					return callback?.(error);
				}
				callback?.(null);
			});
		});
	}

	/**
	 * Safety: This method is only valid to call if the previous rebuild task is finished, or there will be data races.
	 * @deprecated This is a low-level incremental rebuild API, which shouldn't be used intentionally. Use `compiler.build` instead.
	 */
	rebuild(
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
		this.compilation = compilation;
		return compilation;
	}

	#resetThisCompilation() {
		// reassign new compilation in thisCompilation
		this.compilation = undefined;
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
		this.compilationParams = params;
		return params;
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

			this.build(err => {
				if (err) {
					return callback(err);
				}
				this.compilation!.startTime = startTime;
				this.compilation!.endTime = Date.now();
				this.hooks.afterCompile.callAsync(this.compilation!, err => {
					if (err) {
						return callback(err);
					}
					return callback(null, this.compilation);
				});
			});
		});
	}

	watch(watchOptions: WatchOptions, handler: Callback<Error, Stats>): Watching {
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

	purgeInputFileSystem() {
		if (this.inputFileSystem && this.inputFileSystem.purge) {
			this.inputFileSystem.purge();
		}
	}

	close(callback: (error?: Error | null) => void) {
		if (this.watching) {
			// When there is still an active watching, close this first
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

	getAsset(name: string) {
		let source = this.compilation!.__internal__getAssetSource(name);
		if (!source) {
			return null;
		}
		return source.buffer();
	}

	__internal__registerBuiltinPlugin(plugin: binding.BuiltinPlugin) {
		this.builtinPlugins.push(plugin);
	}

	__internal__getModuleExecutionResult(id: number) {
		return this.#moduleExecutionResultsMap.get(id);
	}
}

export { Compiler };
