import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const CssChunkingPlugin = create(
	BuiltinPluginName.CssChunkingPlugin,
	(strict: boolean): boolean => strict
);
