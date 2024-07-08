import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const ModuleConcatenationPlugin = create(
	BuiltinPluginName.ModuleConcatenationPlugin,
	() => {},
	"compilation"
);
