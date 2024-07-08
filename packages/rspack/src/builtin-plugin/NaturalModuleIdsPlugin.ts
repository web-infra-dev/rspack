import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const NaturalModuleIdsPlugin = create(
	BuiltinPluginName.NaturalModuleIdsPlugin,
	() => {},
	"compilation"
);
