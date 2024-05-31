import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const NaturalChunkIdsPlugin = create(
	BuiltinPluginName.NaturalChunkIdsPlugin,
	() => {},
	"compilation"
);
