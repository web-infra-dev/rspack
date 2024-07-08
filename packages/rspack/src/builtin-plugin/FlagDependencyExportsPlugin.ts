import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const FlagDependencyExportsPlugin = create(
	BuiltinPluginName.FlagDependencyExportsPlugin,
	() => {},
	"compilation"
);
