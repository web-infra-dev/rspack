import { JsParserPluginName } from "@rspack/binding";

import { create } from "./base";

export type DefinePluginOptions = Record<string, string | boolean | undefined>;
export const DefinePlugin = create(
	JsParserPluginName.DefinePlugin,
	(define: DefinePluginOptions): Record<string, string> => {
		const entries = Object.entries(define).map(([key, value]) => {
			if (typeof value !== "string") {
				value = value === undefined ? "undefined" : JSON.stringify(value);
			}
			return [key, value];
		});
		return Object.fromEntries(entries);
	}
);
