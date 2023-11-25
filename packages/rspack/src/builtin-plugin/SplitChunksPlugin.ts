import assert from "assert";
import { toRawSplitChunksOptions } from "../config/adapter";
import { type OptimizationSplitChunksOptions } from "../config/zod";
import { BuiltinPluginName, create } from "./base";

export const SplitChunksPlugin = create(
	BuiltinPluginName.SplitChunksPlugin,
	(options: OptimizationSplitChunksOptions) => {
		let raw = toRawSplitChunksOptions(options);
		assert(typeof raw !== "undefined");
		return raw;
	}
);

export const OldSplitChunksPlugin = create(
	BuiltinPluginName.SplitChunksPlugin,
	(options: OptimizationSplitChunksOptions) => {
		let raw = toRawSplitChunksOptions(options);
		assert(typeof raw !== "undefined");
		return raw;
	}
);
