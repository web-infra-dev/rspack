import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const RemoveParentModulesPlugin = create(
	BuiltinPluginName.RemoveParentModulesPlugin,
	() => {}
);
