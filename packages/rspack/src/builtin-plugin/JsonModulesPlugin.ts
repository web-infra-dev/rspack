import { BuiltinPluginName, create } from "./base";

export const JsonModulesPlugin = create(
	BuiltinPluginName.JsonModulesPlugin,
	() => {},
	"compilation"
);
