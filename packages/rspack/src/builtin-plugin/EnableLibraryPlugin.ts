import { BuiltinPluginName, create } from "./base";

export const EnableLibraryPlugin = create(
	BuiltinPluginName.EnableLibraryPlugin,
	type => type
);
