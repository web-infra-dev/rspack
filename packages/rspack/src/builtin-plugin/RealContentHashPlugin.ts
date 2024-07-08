import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const RealContentHashPlugin = create(
	BuiltinPluginName.RealContentHashPlugin,
	() => {},
	"compilation"
);
