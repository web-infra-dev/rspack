import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const JavascriptModulesPlugin = create(
	BuiltinPluginName.JavascriptModulesPlugin,
	() => {},
	"compilation"
);
