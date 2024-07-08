import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const NodeTargetPlugin = create(
	BuiltinPluginName.NodeTargetPlugin,
	() => undefined
);
