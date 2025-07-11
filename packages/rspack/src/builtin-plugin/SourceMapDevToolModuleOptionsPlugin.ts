import {
	BuiltinPluginName,
	type SourceMapDevToolModuleOptionsPluginOptions
} from "@rspack/binding";

import { create } from "./base";

export type { SourceMapDevToolModuleOptionsPluginOptions };

export const SourceMapDevToolModuleOptionsPlugin = create(
	BuiltinPluginName.SourceMapDevToolModuleOptionsPlugin,
	(options: SourceMapDevToolModuleOptionsPluginOptions) => options
);
