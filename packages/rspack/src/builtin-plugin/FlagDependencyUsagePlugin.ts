import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const FlagDependencyUsagePlugin = create(
	BuiltinPluginName.FlagDependencyUsagePlugin,
	(global: boolean) => global,
	"compilation"
);
