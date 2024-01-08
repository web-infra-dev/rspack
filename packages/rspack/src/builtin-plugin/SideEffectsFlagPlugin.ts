import { BuiltinPluginName, create } from "./base";

export const SideEffectsFlagPlugin = create(
	BuiltinPluginName.SideEffectsFlagPlugin,
	() => {},
	"compilation"
);
