import { BuiltinPluginName, create } from "./base";

export const EnableWasmLoadingPlugin = create(
	BuiltinPluginName.EnableWasmLoadingPlugin,
	type => type
);
