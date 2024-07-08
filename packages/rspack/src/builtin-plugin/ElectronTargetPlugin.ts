import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const ElectronTargetPlugin = create(
	BuiltinPluginName.ElectronTargetPlugin,
	(context?: string) => context ?? "none"
);
