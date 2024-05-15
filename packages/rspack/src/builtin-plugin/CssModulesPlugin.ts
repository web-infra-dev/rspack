import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const CssModulesPlugin = create(
	BuiltinPluginName.CssModulesPlugin,
	() => {},
	"compilation"
);
