import { BuiltinPluginName, create } from "./base";

export const FlagDependencyExportsPlugin = create(
	BuiltinPluginName.FlagDependencyExportsPlugin,
	() => {},
	"compilation"
);
