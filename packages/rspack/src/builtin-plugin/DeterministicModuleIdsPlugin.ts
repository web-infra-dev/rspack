import { BuiltinPluginName, create } from "./base";

export const DeterministicModuleIdsPlugin = create(
	BuiltinPluginName.DeterministicModuleIdsPlugin,
	() => {},
	"compilation"
);
