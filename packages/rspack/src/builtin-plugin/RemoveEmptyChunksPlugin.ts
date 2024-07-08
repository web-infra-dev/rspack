import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const RemoveEmptyChunksPlugin = create(
	BuiltinPluginName.RemoveEmptyChunksPlugin,
	() => {},
	"compilation"
);
