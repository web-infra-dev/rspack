import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const CommonJsChunkFormatPlugin = create(
	BuiltinPluginName.CommonJsChunkFormatPlugin,
	() => {}
);
