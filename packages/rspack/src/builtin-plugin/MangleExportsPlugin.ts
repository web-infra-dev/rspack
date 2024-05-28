import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const MangleExportsPlugin = create(
	BuiltinPluginName.MangleExportsPlugin,
	(deterministic: boolean) => deterministic,
	"compilation"
);
