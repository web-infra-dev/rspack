import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const RSCServerReferenceManifestRspackPlugin = create(
	BuiltinPluginName.RSCServerReferenceManifestRspackPlugin,
	() => {},
	"compilation"
);
