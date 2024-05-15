import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const SideEffectsFlagPlugin = create(
	BuiltinPluginName.SideEffectsFlagPlugin,
	() => {},
	"compilation"
);
