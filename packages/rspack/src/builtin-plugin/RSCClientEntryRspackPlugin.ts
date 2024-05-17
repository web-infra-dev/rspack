import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const RSCClientEntryRspackPlugin = create(
	BuiltinPluginName.RSCClientEntryRspackPlugin,
	options => options,
	"compilation"
);
