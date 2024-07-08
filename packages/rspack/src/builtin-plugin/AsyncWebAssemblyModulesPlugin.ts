import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const AsyncWebAssemblyModulesPlugin = create(
	BuiltinPluginName.AsyncWebAssemblyModulesPlugin,
	() => {},
	"compilation"
);
