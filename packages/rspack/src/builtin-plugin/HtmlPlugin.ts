import { z } from "zod";
import { RawHtmlPluginConfig } from "@rspack/binding";
import { BuiltinPluginKind, create } from "./base";
import { validate } from "../util/validate";

const htmlPluginOptions = z.strictObject({
	filename: z.string().optional(),
	template: z.string().optional(),
	templateContent: z.string().optional(),
	templateParameters: z.record(z.string()).optional(),
	inject: z.enum(["head", "body"]).optional(),
	publicPath: z.string().optional(),
	scriptLoading: z.enum(["blocking", "defer", "module"]).optional(),
	chunks: z.string().array().optional(),
	excludedChunks: z.string().array().optional(),
	sri: z.enum(["sha256", "sha384", "sha512"]).optional(),
	minify: z.boolean().optional(),
	title: z.string().optional(),
	favicon: z.string().optional(),
	meta: z.record(z.string().or(z.record(z.string()))).optional()
});
export type HtmlPluginOptions = z.infer<typeof htmlPluginOptions>;
export const HtmlPlugin = create(
	BuiltinPluginKind.Html,
	(c: HtmlPluginOptions): RawHtmlPluginConfig => {
		validate(c, htmlPluginOptions);
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
