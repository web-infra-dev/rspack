import { BuiltinPluginName, create } from "./base";

export const AsyncWebAssemblyModulesPlugin = create(
	BuiltinPluginName.AsyncWebAssemblyModulesPlugin,
	() => {},
	"compilation"
);
