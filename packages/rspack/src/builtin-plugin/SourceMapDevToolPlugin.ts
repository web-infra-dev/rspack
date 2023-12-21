import { RawSourceMapDevToolPluginOptions } from "@rspack/binding";
import { BuiltinPluginName, create } from "./base";

export type SourceMapDevToolPluginOptions = {
	filename?: false | null | string;
	append?: boolean;
	namespace?: string;
	columns?: boolean;
	noSources?: boolean;
	publicPath?: string;
};

export const SourceMapDevToolPlugin = create(
	BuiltinPluginName.SourceMapDevToolPlugin,
	(
		options: SourceMapDevToolPluginOptions
	): RawSourceMapDevToolPluginOptions => {
		return {
			filename: options.filename || undefined,
			append: options.append,
			namespace: options.namespace ?? "",
			columns: options.columns ?? true,
			noSources: options.noSources ?? false,
			publicPath: options.publicPath
		};
	},
	"compilation"
);
