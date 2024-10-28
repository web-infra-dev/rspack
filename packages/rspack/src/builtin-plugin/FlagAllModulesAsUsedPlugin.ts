import { BuiltinPluginName } from "@rspack/binding";
import { create } from "./base";

export const FlagAllModulesAsUsedPlugin = create(
	BuiltinPluginName.FlagAllModulesAsUsedPlugin,
	(): undefined => {}
);
