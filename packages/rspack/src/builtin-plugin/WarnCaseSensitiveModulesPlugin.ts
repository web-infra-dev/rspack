import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const WarnCaseSensitiveModulesPlugin = create(
	BuiltinPluginName.WarnCaseSensitiveModulesPlugin,
	() => {},
	"compilation"
);
