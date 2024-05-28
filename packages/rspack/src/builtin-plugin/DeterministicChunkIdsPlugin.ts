import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const DeterministicChunkIdsPlugin = create(
	BuiltinPluginName.DeterministicChunkIdsPlugin,
	() => {},
	"compilation"
);
