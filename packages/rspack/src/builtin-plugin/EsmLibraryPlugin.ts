import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const EsmLibraryPlugin = create(
	BuiltinPluginName.EsmLibraryPlugin,
	() => {}
);
