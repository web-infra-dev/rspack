import assert from "assert";
import {
	BuiltinPlugin,
	BuiltinPluginName,
	JsChunk,
	JsModule,
	RawCacheGroupOptions,
	RawSplitChunksOptions
} from "@rspack/binding";

import { Chunk } from "../Chunk";
import { Compiler } from "../Compiler";
import { Module } from "../Module";
import { type OptimizationSplitChunksOptions } from "../config/zod";
import { JsSplitChunkSizes } from "../util/SplitChunkSize";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";

export class SplitChunksPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.SplitChunksPlugin;
	affectedHooks = "thisCompilation" as const;

	constructor(private options: OptimizationSplitChunksOptions) {
		super();
	}

	raw(compiler: Compiler): BuiltinPlugin {
		const rawOptions = toRawSplitChunksOptions(this.options, compiler);
		assert(typeof rawOptions !== "undefined");
		return createBuiltinPlugin(this.name, rawOptions);
	}
}

function toRawSplitChunksOptions(
	sc: false | OptimizationSplitChunksOptions,
	compiler: Compiler
): RawSplitChunksOptions | undefined {
	if (!sc) {
		return;
	}

	function getName(name: any) {
		interface Context {
			module: JsModule;
			chunks: JsChunk[];
			cacheGroupKey: string;
		}

		if (typeof name === "function") {
			return (ctx: Context) => {
				if (typeof ctx.module === "undefined") {
					return name(undefined);
				} else {
					return name(
						Module.__from_binding(ctx.module, compiler._lastCompilation),
						getChunks(ctx.chunks),
						ctx.cacheGroupKey
					);
				}
			};
		} else {
			return name;
		}
	}

	function getTest(test: any) {
		interface Context {
			module: JsModule;
		}

		if (typeof test === "function") {
			return (ctx: Context) => {
				if (typeof ctx.module === "undefined") {
					return test(undefined);
				} else {
					return test(
						Module.__from_binding(ctx.module, compiler._lastCompilation)
					);
				}
			};
		} else {
			return test;
		}
	}

	function getChunks(chunks: any) {
		if (typeof chunks === "function") {
			return (chunk: JsChunk) =>
				chunks(
					Chunk.__from_binding(
						chunk,
						compiler._lastCompilation!.__internal_getInner()
					)
				);
		} else {
			return chunks;
		}
	}

	const {
		name,
		chunks,
		defaultSizeTypes,
		cacheGroups = {},
		fallbackCacheGroup,
		minSize,
		maxSize,
		maxAsyncSize,
		maxInitialSize,
		...passThrough
	} = sc;

	return {
		name: getName(name),
		chunks: getChunks(chunks),
		defaultSizeTypes: defaultSizeTypes || ["javascript", "unknown"],
		cacheGroups: Object.entries(cacheGroups)
			.filter(([_key, group]) => group !== false)
			.map(([key, group]) => {
				group = group as Exclude<typeof group, false>;

				const {
					test,
					name,
					chunks,
					minSize,
					maxSize,
					maxAsyncSize,
					maxInitialSize,
					...passThrough
				} = group;
				const rawGroup: RawCacheGroupOptions = {
					key,
					test: getTest(test),
					name: getName(name),
					chunks: getChunks(chunks),
					minSize: JsSplitChunkSizes.__to_binding(minSize),
					maxSize: JsSplitChunkSizes.__to_binding(maxSize),
					maxAsyncSize: JsSplitChunkSizes.__to_binding(maxAsyncSize),
					maxInitialSize: JsSplitChunkSizes.__to_binding(maxInitialSize),
					...passThrough
				};
				return rawGroup;
			}),
		fallbackCacheGroup: {
			chunks: getChunks(chunks),
			...fallbackCacheGroup
		},
		minSize: JsSplitChunkSizes.__to_binding(minSize),
		maxSize: JsSplitChunkSizes.__to_binding(maxSize),
		maxAsyncSize: JsSplitChunkSizes.__to_binding(maxAsyncSize),
		maxInitialSize: JsSplitChunkSizes.__to_binding(maxInitialSize),
		...passThrough
	};
}
