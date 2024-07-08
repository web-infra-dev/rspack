import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const NamedChunkIdsPlugin = create(
	BuiltinPluginName.NamedChunkIdsPlugin,
	() => {},
	"compilation"
);
