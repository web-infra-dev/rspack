import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const NamedModuleIdsPlugin = create(
	BuiltinPluginName.NamedModuleIdsPlugin,
	() => {},
	"compilation"
);
