import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const EnableWasmLoadingPlugin = create(
	BuiltinPluginName.EnableWasmLoadingPlugin,
	type => type
);
