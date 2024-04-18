import { BuiltinPluginName } from "@rspack/binding";
import { create } from "../builtin-plugin/base";

export const FederationRuntimePlugin = create(
	BuiltinPluginName.FederationRuntimePlugin,
	() => {}
);
