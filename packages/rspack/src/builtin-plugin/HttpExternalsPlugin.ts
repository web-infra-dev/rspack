import { RawHttpExternalsPluginOptions } from "@rspack/binding";
import { BuiltinPluginKind, create } from "./base";

export const HttpExternalsPlugin = create(
	BuiltinPluginKind.HttpExternals,
	(css: boolean): RawHttpExternalsPluginOptions => {
		return {
			css
		};
	}
);
