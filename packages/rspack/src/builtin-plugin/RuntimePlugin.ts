import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const RuntimePlugin = create(
	BuiltinPluginName.RuntimePlugin,
	() => {},
	"compilation"
);
