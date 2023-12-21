import { BuiltinPluginName, create } from "./base";

export const WarnCaseSensitiveModulesPlugin = create(
	BuiltinPluginName.WarnCaseSensitiveModulesPlugin,
	() => {},
	"compilation"
);
