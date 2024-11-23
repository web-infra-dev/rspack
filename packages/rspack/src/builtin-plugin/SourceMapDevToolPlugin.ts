import {
	BuiltinPluginName,
	type RawSourceMapDevToolPluginOptions
} from "@rspack/binding";

import { create } from "./base";

export type { RawSourceMapDevToolPluginOptions as SourceMapDevToolPluginOptions };

export const SourceMapDevToolPlugin = create(
	BuiltinPluginName.SourceMapDevToolPlugin,
	(options: RawSourceMapDevToolPluginOptions) => options,
	"compilation"
);
