import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const RSCClientEntryPlugin = create(
	BuiltinPluginName.RSCClientEntryRspackPlugin,
	options => options,
	"compilation"
);
