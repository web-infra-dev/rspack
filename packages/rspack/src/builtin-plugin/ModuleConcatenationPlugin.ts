import { BuiltinPluginName, create } from "./base";

export const ModuleConcatenationPlugin = create(
	BuiltinPluginName.ModuleConcatenationPlugin,
	() => {},
	"compilation"
);
