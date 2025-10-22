import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import { createBuiltinPlugin, RspackBuiltinPlugin } from "./base";

export type RenameExportsPluginOptions = {
	mangleExports: boolean | "deterministic" | "size";
	inlineExports: boolean;
};

export class RenameExportsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.RenameExportsPlugin;
	affectedHooks = "compilation" as const;

	constructor(private options: RenameExportsPluginOptions) {
		super();
	}

	raw(): BuiltinPlugin {
		return createBuiltinPlugin(this.name, this.options);
	}
}
