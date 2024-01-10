import { BuiltinPluginName, create } from "./base";

export const MangleExportsPlugin = create(
	BuiltinPluginName.MangleExportsPlugin,
	(deterministic: boolean) => deterministic,
	"compilation"
);
