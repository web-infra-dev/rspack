import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const JsonModulesPlugin = create(
	BuiltinPluginName.JsonModulesPlugin,
	() => {},
	"compilation"
);
