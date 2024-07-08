import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const ChunkPrefetchPreloadPlugin = create(
	BuiltinPluginName.ChunkPrefetchPreloadPlugin,
	() => {}
);
