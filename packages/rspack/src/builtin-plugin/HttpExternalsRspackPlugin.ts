import {
	BuiltinPluginName,
	RawHttpExternalsRspackPluginOptions
} from "@rspack/binding";

import { create } from "./base";

export const HttpExternalsRspackPlugin = create(
	BuiltinPluginName.HttpExternalsRspackPlugin,
	(css: boolean, webAsync: boolean): RawHttpExternalsRspackPluginOptions => {
		return {
			css,
			webAsync
		};
	}
);
