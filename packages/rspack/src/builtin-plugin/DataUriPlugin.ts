import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const DataUriPlugin = create(
	BuiltinPluginName.DataUriPlugin,
	() => {},
	"compilation"
);
