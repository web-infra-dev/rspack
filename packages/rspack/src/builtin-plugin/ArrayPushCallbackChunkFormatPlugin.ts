import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const ArrayPushCallbackChunkFormatPlugin = create(
	BuiltinPluginName.ArrayPushCallbackChunkFormatPlugin,
	() => {}
);
