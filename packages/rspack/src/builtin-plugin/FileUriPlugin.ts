import { BuiltinPluginName, create } from "./base";

export const FileUriPlugin = create(
	BuiltinPluginName.FileUriPlugin,
	() => {},
	"compilation"
);
