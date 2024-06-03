import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const RSCClientReferenceManifestRspackPlugin = create(
	BuiltinPluginName.RSCClientReferenceManifestRspackPlugin,
	() => {},
	"compilation"
);
