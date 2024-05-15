import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const MergeDuplicateChunksPlugin = create(
	BuiltinPluginName.MergeDuplicateChunksPlugin,
	() => {}
);
