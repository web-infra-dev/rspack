import { RawHttpExternalsRspackPluginOptions } from "@rspack/binding";
import { BuiltinPluginName, create } from "./base";

export const HttpExternalsRspackPlugin = create(
	BuiltinPluginName.HttpExternalsRspackPlugin,
	(css: boolean): RawHttpExternalsRspackPluginOptions => {
		return {
			css
		};
	}
);
