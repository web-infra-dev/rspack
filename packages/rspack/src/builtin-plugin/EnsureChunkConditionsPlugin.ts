import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const EnsureChunkConditionsPlugin = create(
	BuiltinPluginName.EnsureChunkConditionsPlugin,
	() => {}
);
