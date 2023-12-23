import { RawSourceMapDevToolPluginOptions } from "@rspack/binding";
import { BuiltinPluginName, create } from "./base";

export type SourceMapDevToolPluginOptions = RawSourceMapDevToolPluginOptions;

export const SourceMapDevToolPlugin = create(
	BuiltinPluginName.SourceMapDevToolPlugin,
	(
		options: SourceMapDevToolPluginOptions
	): RawSourceMapDevToolPluginOptions => {
		return options;
	}
);
