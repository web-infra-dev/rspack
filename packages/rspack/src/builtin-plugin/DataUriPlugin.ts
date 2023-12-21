import { BuiltinPluginName, create } from "./base";

export const DataUriPlugin = create(
	BuiltinPluginName.DataUriPlugin,
	() => {},
	"compilation"
);
