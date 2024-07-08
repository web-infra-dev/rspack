import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const DeterministicModuleIdsPlugin = create(
	BuiltinPluginName.DeterministicModuleIdsPlugin,
	() => {},
	"compilation"
);
