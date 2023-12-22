import { BuiltinPluginName, create } from "./base";

export const NaturalModuleIdsPlugin = create(
	BuiltinPluginName.NaturalModuleIdsPlugin,
	() => {},
	"compilation"
);
