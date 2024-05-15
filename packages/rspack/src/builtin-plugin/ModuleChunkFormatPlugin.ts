import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const ModuleChunkFormatPlugin = create(
	BuiltinPluginName.ModuleChunkFormatPlugin,
	() => {}
);
