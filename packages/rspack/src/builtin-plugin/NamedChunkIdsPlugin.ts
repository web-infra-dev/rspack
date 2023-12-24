import { BuiltinPluginName, create } from "./base";

export const NamedChunkIdsPlugin = create(
	BuiltinPluginName.NamedChunkIdsPlugin,
	() => {},
	"compilation"
);
