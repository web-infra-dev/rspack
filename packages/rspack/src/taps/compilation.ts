import * as binding from "@rspack/binding";
import { Chunk } from "../Chunk";
import { CodeGenerationResult, type Module } from "../Module";
import {
	RuntimeGlobals,
	__from_binding_runtime_globals,
	__to_binding_runtime_globals
} from "../RuntimeGlobals";
import { tryRunOrWebpackError } from "../lib/HookWebpackError";
import { toBuffer } from "../util";
import { createHash } from "../util/createHash";
import type { CreatePartialRegisters } from "./types";

export const createCompilationHooksRegisters: CreatePartialRegisters<
	`Compilation`
> = (getCompiler, createTap, createMapTap) => {
	return {
		registerCompilationAdditionalTreeRuntimeRequirementsTaps: createTap(
			binding.RegisterJsTapKind.CompilationAdditionalTreeRuntimeRequirements,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks
					.additionalTreeRuntimeRequirements;
			},

			function (queried) {
				return function ({
					chunk,
					runtimeRequirements
				}: binding.JsAdditionalTreeRuntimeRequirementsArg) {
					const set = __from_binding_runtime_globals(runtimeRequirements);
					queried.call(Chunk.__from_binding(chunk), set);
					return {
						runtimeRequirements: __to_binding_runtime_globals(set)
					};
				};
			}
		),
		registerCompilationRuntimeRequirementInTreeTaps: createMapTap(
			binding.RegisterJsTapKind.CompilationRuntimeRequirementInTree,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks
					.runtimeRequirementInTree;
			},

			function (queried) {
				return function ({
					chunk: chunkBinding,
					runtimeRequirements
				}: binding.JsRuntimeRequirementInTreeArg) {
					const set = __from_binding_runtime_globals(runtimeRequirements);
					const chunk = Chunk.__from_binding(chunkBinding);
					for (const r of set) {
						queried.for(r).call(chunk, set);
					}
					return {
						runtimeRequirements: __to_binding_runtime_globals(set)
					};
				};
			}
		),
		registerCompilationRuntimeModuleTaps: createTap(
			binding.RegisterJsTapKind.CompilationRuntimeModule,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks.runtimeModule;
			},

			function (queried) {
				return function ({ module, chunk }: binding.JsRuntimeModuleArg) {
					const originSource = module.source?.source;
					queried.call(module, Chunk.__from_binding(chunk));
					const newSource = module.source?.source;
					if (newSource && newSource !== originSource) {
						return module;
					}
					return;
				};
			}
		),
		registerCompilationBuildModuleTaps: createTap(
			binding.RegisterJsTapKind.CompilationBuildModule,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks.buildModule;
			},

			function (queried) {
				return function (module: Module) {
					return queried.call(module);
				};
			}
		),
		registerCompilationStillValidModuleTaps: createTap(
			binding.RegisterJsTapKind.CompilationStillValidModule,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks
					.stillValidModule;
			},

			function (queried) {
				return function (module: Module) {
					return queried.call(module);
				};
			}
		),
		registerCompilationSucceedModuleTaps: createTap(
			binding.RegisterJsTapKind.CompilationSucceedModule,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks.succeedModule;
			},

			function (queried) {
				return function (module: Module) {
					return queried.call(module);
				};
			}
		),
		registerCompilationExecuteModuleTaps: createTap(
			binding.RegisterJsTapKind.CompilationExecuteModule,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks.executeModule;
			},

			function (queried) {
				return function ({
					entry,
					id,
					codegenResults,
					runtimeModules
				}: binding.JsExecuteModuleArg) {
					try {
						const __webpack_require__: any = (id: string) => {
							const cached = moduleCache[id];
							if (cached !== undefined) {
								if (cached.error) throw cached.error;
								return cached.exports;
							}

							const execOptions = {
								id,
								module: {
									id,
									exports: {},
									loaded: false,
									error: undefined
								},
								require: __webpack_require__
							};

							for (const handler of interceptModuleExecution) {
								handler(execOptions);
							}

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
						const interceptModuleExecution: ((execOptions: any) => void)[] =
							(__webpack_require__[
								RuntimeGlobals.interceptModuleExecution.replace(
									`${RuntimeGlobals.require}.`,
									""
								)
							] = []);

						for (const runtimeModule of runtimeModules) {
							__webpack_require__(runtimeModule);
						}

						const executeResult = __webpack_require__(entry);
						getCompiler()
							.__internal__get_module_execution_results_map()
							.set(id, executeResult);
					} catch (e) {
						getCompiler()
							.__internal__get_module_execution_results_map()
							.set(id, e);
						throw e;
					}
				};
			}
		),
		registerCompilationFinishModulesTaps: createTap(
			binding.RegisterJsTapKind.CompilationFinishModules,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks.finishModules;
			},

			function (queried) {
				return async function () {
					return await queried.promise(
						getCompiler().__internal__get_compilation()!.modules
					);
				};
			}
		),
		registerCompilationOptimizeModulesTaps: createTap(
			binding.RegisterJsTapKind.CompilationOptimizeModules,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks
					.optimizeModules;
			},

			function (queried) {
				return function () {
					return queried.call(
						getCompiler().__internal__get_compilation()!.modules.values()
					);
				};
			}
		),
		registerCompilationAfterOptimizeModulesTaps: createTap(
			binding.RegisterJsTapKind.CompilationAfterOptimizeModules,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks
					.afterOptimizeModules;
			},

			function (queried) {
				return function () {
					queried.call(
						getCompiler().__internal__get_compilation()!.modules.values()
					);
				};
			}
		),
		registerCompilationOptimizeTreeTaps: createTap(
			binding.RegisterJsTapKind.CompilationOptimizeTree,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks.optimizeTree;
			},

			function (queried) {
				return async function () {
					return await queried.promise(
						getCompiler().__internal__get_compilation()!.chunks,
						getCompiler().__internal__get_compilation()!.modules
					);
				};
			}
		),
		registerCompilationOptimizeChunkModulesTaps: createTap(
			binding.RegisterJsTapKind.CompilationOptimizeChunkModules,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks
					.optimizeChunkModules;
			},

			function (queried) {
				return async function () {
					return await queried.promise(
						getCompiler().__internal__get_compilation()!.chunks,
						getCompiler().__internal__get_compilation()!.modules
					);
				};
			}
		),
		registerCompilationChunkHashTaps: createTap(
			binding.RegisterJsTapKind.CompilationChunkHash,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks.chunkHash;
			},

			function (queried) {
				return function (chunk: binding.JsChunk) {
					if (!getCompiler().options.output.hashFunction) {
						throw new Error("'output.hashFunction' cannot be undefined");
					}
					const hash = createHash(getCompiler().options.output.hashFunction!);
					queried.call(Chunk.__from_binding(chunk), hash);
					let digestResult: Buffer | string;
					if (getCompiler().options.output.hashDigest) {
						digestResult = hash.digest(
							getCompiler().options.output.hashDigest as string
						);
					} else {
						digestResult = hash.digest();
					}
					return toBuffer(digestResult);
				};
			}
		),
		registerCompilationChunkAssetTaps: createTap(
			binding.RegisterJsTapKind.CompilationChunkAsset,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks.chunkAsset;
			},

			function (queried) {
				return function ({ chunk, filename }: binding.JsChunkAssetArgs) {
					return queried.call(Chunk.__from_binding(chunk), filename);
				};
			}
		),
		registerCompilationProcessAssetsTaps: createTap(
			binding.RegisterJsTapKind.CompilationProcessAssets,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks.processAssets;
			},

			function (queried) {
				return async function () {
					return await queried.promise(
						getCompiler().__internal__get_compilation()!.assets
					);
				};
			}
		),
		registerCompilationAfterProcessAssetsTaps: createTap(
			binding.RegisterJsTapKind.CompilationAfterProcessAssets,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks
					.afterProcessAssets;
			},

			function (queried) {
				return function () {
					return queried.call(
						getCompiler().__internal__get_compilation()!.assets
					);
				};
			}
		),
		registerCompilationSealTaps: createTap(
			binding.RegisterJsTapKind.CompilationSeal,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks.seal;
			},

			function (queried) {
				return function () {
					return queried.call();
				};
			}
		),
		registerCompilationAfterSealTaps: createTap(
			binding.RegisterJsTapKind.CompilationAfterSeal,

			function () {
				return getCompiler().__internal__get_compilation()!.hooks.afterSeal;
			},

			function (queried) {
				return async function () {
					return await queried.promise();
				};
			}
		)
	};
};
