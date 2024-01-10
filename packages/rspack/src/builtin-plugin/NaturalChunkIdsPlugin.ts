import { BuiltinPluginName, create } from "./base";

export const NaturalChunkIdsPlugin = create(
	BuiltinPluginName.NaturalChunkIdsPlugin,
	() => {},
	"compilation"
);
