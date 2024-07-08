import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const EnableChunkLoadingPlugin = create(
	BuiltinPluginName.EnableChunkLoadingPlugin,
	type => type
);
