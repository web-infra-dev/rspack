import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const InferAsyncModulesPlugin = create(
	BuiltinPluginName.InferAsyncModulesPlugin,
	() => {},
	"compilation"
);
