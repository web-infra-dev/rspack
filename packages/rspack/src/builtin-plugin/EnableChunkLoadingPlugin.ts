import { BuiltinPluginName, create } from "./base";

export const EnableChunkLoadingPlugin = create(
	BuiltinPluginName.EnableChunkLoadingPlugin,
	type => type
);
