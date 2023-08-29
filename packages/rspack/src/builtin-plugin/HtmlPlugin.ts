import { RawHtmlPluginConfig } from "@rspack/binding";
import { BuiltinPluginKind, create } from "./base";

export type HtmlPluginOptions = Omit<RawHtmlPluginConfig, "meta"> & {
	meta?: Record<string, string | Record<string, string>>;
};
export const HtmlPlugin = create(
	BuiltinPluginKind.Html,
	(c: HtmlPluginOptions): RawHtmlPluginConfig => {
		const meta: Record<string, Record<string, string>> = {};
		for (const key in c.meta) {
			const value = c.meta[key];
			if (typeof value === "string") {
				meta[key] = {
					name: key,
					content: value
				};
			}
		}
		return {
			...c,
			meta
		};
	}
);
