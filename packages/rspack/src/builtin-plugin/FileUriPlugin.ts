import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const FileUriPlugin = create(
	BuiltinPluginName.FileUriPlugin,
	() => {},
	"compilation"
);
