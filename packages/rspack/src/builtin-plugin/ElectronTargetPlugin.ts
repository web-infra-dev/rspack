import { BuiltinPluginName, create } from "./base";

export const ElectronTargetPlugin = create(
	BuiltinPluginName.ElectronTargetPlugin,
	(context?: string) => context ?? "none"
);
