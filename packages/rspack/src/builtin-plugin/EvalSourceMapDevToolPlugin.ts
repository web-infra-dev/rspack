import {
	BuiltinPluginName,
	RawSourceMapDevToolPluginOptions
} from "@rspack/binding";

import { SourceMapDevToolPluginOptions } from "./SourceMapDevToolPlugin";
import { create } from "./base";

export const EvalSourceMapDevToolPlugin = create(
	BuiltinPluginName.EvalSourceMapDevToolPlugin,
	(
		options: SourceMapDevToolPluginOptions
	): RawSourceMapDevToolPluginOptions => {
		return {
			filename: options.filename || undefined,
			append: options.append,
			namespace: options.namespace ?? "",
			columns: options.columns ?? true,
			noSources: options.noSources ?? false,
			publicPath: options.publicPath,
			module: options.module
		};
	},
	"compilation"
);
