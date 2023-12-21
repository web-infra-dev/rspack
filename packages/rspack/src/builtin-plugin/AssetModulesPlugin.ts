import { BuiltinPluginName, create } from "./base";

export const AssetModulesPlugin = create(
	BuiltinPluginName.AssetModulesPlugin,
	() => {},
	"compilation"
);
