import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const AssetModulesPlugin = create(
	BuiltinPluginName.AssetModulesPlugin,
	() => {},
	"compilation"
);
