import { BuiltinPluginName } from "@rspack/binding";
import { create } from "./base";

export const EnableLibraryPlugin = create(
	BuiltinPluginName.EnableLibraryPlugin,
	type => type
);
