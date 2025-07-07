import {
	BuiltinPluginName,
	type JsRsdoctorAsset,
	type JsRsdoctorAssetPatch,
	type JsRsdoctorChunk,
	type JsRsdoctorChunkAssets,
	type JsRsdoctorChunkGraph,
	type JsRsdoctorChunkModules,
	type JsRsdoctorDependency,
	type JsRsdoctorEntrypoint,
	type JsRsdoctorEntrypointAssets,
	type JsRsdoctorExportInfo,
	type JsRsdoctorModule,
	type JsRsdoctorModuleGraph,
	type JsRsdoctorModuleGraphModule,
	type JsRsdoctorModuleIdsPatch,
	type JsRsdoctorModuleOriginalSource,
	type JsRsdoctorModuleSourcesPatch,
	type JsRsdoctorSideEffect,
	type JsRsdoctorSourcePosition,
	type JsRsdoctorSourceRange,
	type JsRsdoctorStatement,
	type JsRsdoctorVariable,
	type RawRsdoctorPluginOptions,
	RegisterJsTapKind
} from "@rspack/binding";
import * as liteTapable from "@rspack/lite-tapable";
import { type Compilation, checkCompilation } from "../Compilation";
import type { Compiler } from "../Compiler";
import { z } from "../config/zod";
import type { CreatePartialRegisters } from "../taps/types";
import { memoize } from "../util/memoize";
import { validate } from "../util/validate";
import { create } from "./base";

export declare namespace RsdoctorPluginData {
	export type {
		JsRsdoctorAsset as RsdoctorAsset,
		JsRsdoctorChunkGraph as RsdoctorChunkGraph,
		JsRsdoctorModuleGraph as RsdoctorModuleGraph,
		JsRsdoctorChunk as RsdoctorChunk,
		JsRsdoctorModule as RsdoctorModule,
		JsRsdoctorSideEffect as RsdoctorSideEffect,
		JsRsdoctorExportInfo as RsdoctorExportInfo,
		JsRsdoctorVariable as RsdoctorVariable,
		JsRsdoctorDependency as RsdoctorDependency,
		JsRsdoctorEntrypoint as RsdoctorEntrypoint,
		JsRsdoctorStatement as RsdoctorStatement,
		JsRsdoctorSourceRange as RsdoctorSourceRange,
		JsRsdoctorSourcePosition as RsdoctorSourcePosition,
		JsRsdoctorModuleGraphModule as RsdoctorModuleGraphModule,
		JsRsdoctorModuleIdsPatch as RsdoctorModuleIdsPatch,
		JsRsdoctorModuleOriginalSource as RsdoctorModuleOriginalSource,
		JsRsdoctorAssetPatch as RsdoctorAssetPatch,
		JsRsdoctorChunkAssets as RsdoctorChunkAssets,
		JsRsdoctorEntrypointAssets as RsdoctorEntrypointAssets,
		JsRsdoctorChunkModules as RsdoctorChunkModules,
		JsRsdoctorModuleSourcesPatch as RsdoctorModuleSourcesPatch
	};
}

export type RsdoctorPluginOptions = {
	moduleGraphFeatures?: boolean | Array<"graph" | "ids" | "sources">;
	chunkGraphFeatures?: boolean | Array<"graph" | "assets">;
};

const getRsdoctorPluginSchema = memoize(
	() =>
		z.strictObject({
			moduleGraphFeatures: z
				.union([z.boolean(), z.array(z.enum(["graph", "ids", "sources"]))])
				.optional(),
			chunkGraphFeatures: z
				.union([z.boolean(), z.array(z.enum(["graph", "assets"]))])
				.optional()
		}) satisfies z.ZodType<RsdoctorPluginOptions>
);

const RsdoctorPluginImpl = create(
	BuiltinPluginName.RsdoctorPlugin,
	function (
		this: Compiler,
		c: RsdoctorPluginOptions = {
			moduleGraphFeatures: true,
			chunkGraphFeatures: true
		}
	): RawRsdoctorPluginOptions {
		validate(c, getRsdoctorPluginSchema);
		return {
			moduleGraphFeatures: c.moduleGraphFeatures ?? true,
			chunkGraphFeatures: c.chunkGraphFeatures ?? true
		};
	}
);

export type RsdoctorPluginHooks = {
	moduleGraph: liteTapable.AsyncSeriesBailHook<
		[JsRsdoctorModuleGraph],
		false | void
	>;
	chunkGraph: liteTapable.AsyncSeriesBailHook<
		[JsRsdoctorChunkGraph],
		false | void
	>;
	moduleIds: liteTapable.AsyncSeriesBailHook<
		[JsRsdoctorModuleIdsPatch],
		false | void
	>;
	moduleSources: liteTapable.AsyncSeriesBailHook<
		[JsRsdoctorModuleSourcesPatch],
		false | void
	>;
	assets: liteTapable.AsyncSeriesBailHook<[JsRsdoctorAssetPatch], false | void>;
};

const compilationHooksMap: WeakMap<Compilation, RsdoctorPluginHooks> =
	new WeakMap();

const RsdoctorPlugin = RsdoctorPluginImpl as typeof RsdoctorPluginImpl & {
	/**
	 * @deprecated Use `getCompilationHooks` instead.
	 */
	getHooks: (compilation: Compilation) => RsdoctorPluginHooks;
	getCompilationHooks: (compilation: Compilation) => RsdoctorPluginHooks;
};

RsdoctorPlugin.getHooks = RsdoctorPlugin.getCompilationHooks = (
	compilation: Compilation
) => {
	checkCompilation(compilation);

	let hooks = compilationHooksMap.get(compilation);
	if (hooks === undefined) {
		hooks = {
			moduleGraph: new liteTapable.AsyncSeriesBailHook<
				[JsRsdoctorModuleGraph],
				false | void
			>(["moduleGraph"]),
			chunkGraph: new liteTapable.AsyncSeriesBailHook<
				[JsRsdoctorChunkGraph],
				false | void
			>(["chunkGraph"]),
			moduleIds: new liteTapable.AsyncSeriesBailHook<
				[JsRsdoctorModuleIdsPatch],
				false | void
			>(["moduleIdsPatch"]),
			moduleSources: new liteTapable.AsyncSeriesBailHook<
				[JsRsdoctorModuleSourcesPatch],
				false | void
			>(["moduleSourcesPatch"]),
			assets: new liteTapable.AsyncSeriesBailHook<
				[JsRsdoctorAssetPatch],
				false | void
			>(["assetPatch"])
		};
		compilationHooksMap.set(compilation, hooks);
	}
	return hooks;
};

export const createRsdoctorPluginHooksRegisters: CreatePartialRegisters<
	`RsdoctorPlugin`
> = (getCompiler, createTap, createMapTap) => {
	return {
		registerRsdoctorPluginModuleGraphTaps: createTap(
			RegisterJsTapKind.RsdoctorPluginModuleGraph,
			function () {
				return RsdoctorPlugin.getCompilationHooks(
					getCompiler().__internal__get_compilation()!
				).moduleGraph;
			},
			function (queried) {
				return async function (data: JsRsdoctorModuleGraph) {
					return await queried.promise(data);
				};
			}
		),
		registerRsdoctorPluginChunkGraphTaps: createTap(
			RegisterJsTapKind.RsdoctorPluginChunkGraph,
			function () {
				return RsdoctorPlugin.getCompilationHooks(
					getCompiler().__internal__get_compilation()!
				).chunkGraph;
			},
			function (queried) {
				return async function (data: JsRsdoctorChunkGraph) {
					return await queried.promise(data);
				};
			}
		),
		registerRsdoctorPluginModuleIdsTaps: createTap(
			RegisterJsTapKind.RsdoctorPluginModuleIds,
			function () {
				return RsdoctorPlugin.getCompilationHooks(
					getCompiler().__internal__get_compilation()!
				).moduleIds;
			},
			function (queried) {
				return async function (data: JsRsdoctorModuleIdsPatch) {
					return await queried.promise(data);
				};
			}
		),
		registerRsdoctorPluginModuleSourcesTaps: createTap(
			RegisterJsTapKind.RsdoctorPluginModuleSources,
			function () {
				return RsdoctorPlugin.getCompilationHooks(
					getCompiler().__internal__get_compilation()!
				).moduleSources;
			},
			function (queried) {
				return async function (data: JsRsdoctorModuleSourcesPatch) {
					return await queried.promise(data);
				};
			}
		),
		registerRsdoctorPluginAssetsTaps: createTap(
			RegisterJsTapKind.RsdoctorPluginAssets,
			function () {
				return RsdoctorPlugin.getCompilationHooks(
					getCompiler().__internal__get_compilation()!
				).assets;
			},
			function (queried) {
				return async function (data: JsRsdoctorAssetPatch) {
					return await queried.promise(data);
				};
			}
		)
	};
};

export { RsdoctorPlugin };
