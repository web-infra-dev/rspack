import { BuiltinPluginKind, create } from "./base";

export const NodeTargetPlugin = create(
	BuiltinPluginKind.NodeTarget,
	() => undefined
);
