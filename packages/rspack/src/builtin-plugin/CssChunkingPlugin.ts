import { BuiltinPluginName, CssChunkingPluginOptions } from "@rspack/binding";

import { create } from "./base";

export const CssChunkingPlugin = create(
	BuiltinPluginName.CssChunkingPlugin,
	(options: CssChunkingPluginOptions): CssChunkingPluginOptions => options
);
