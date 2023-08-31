import { BuiltinPluginKind, create } from "./base";

export const ElectronTargetPlugin = create(
	BuiltinPluginKind.ElectronTarget,
	(context?: string) => context ?? "none"
);
