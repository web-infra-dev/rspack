import { BuiltinPluginName } from "@rspack/binding";
import { create } from "../builtin-plugin/base";

export const ModuleFederationRuntimePlugin = create(
	BuiltinPluginName.ModuleFederationRuntimePlugin,
	() => {}
);
