import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const ReactServerComponentsPlugin = create(
	BuiltinPluginName.ReactServerComponentsPlugin,
	() => {}
);
