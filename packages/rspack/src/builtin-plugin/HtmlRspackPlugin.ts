import { z } from "zod";
import { BuiltinPluginName, RawHtmlRspackPluginOptions } from "@rspack/binding";
import { create } from "./base";
import { validate } from "../util/validate";

const htmlRspackPluginOptions = z.strictObject({
	filename: z.string().optional(),
	template: z.string().optional(),
	templateContent: z.string().optional(),
	templateParameters: z.record(z.string()).optional(),
	inject: z.enum(["head", "body"]).or(z.boolean()).optional(),
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
export type HtmlRspackPluginOptions = z.infer<typeof htmlRspackPluginOptions>;
export const HtmlRspackPlugin = create(
	BuiltinPluginName.HtmlRspackPlugin,
	(c: HtmlRspackPluginOptions = {}): RawHtmlRspackPluginOptions => {
		validate(c, htmlRspackPluginOptions);
		const meta: Record<string, Record<string, string>> = {};
		for (const key in c.meta) {
			const value = c.meta[key];
			if (typeof value === "string") {
				meta[key] = {
					name: key,
					content: value
				};
			} else {
				meta[key] = {
					name: key,
					...value
				};
			}
		}
		const scriptLoading = c.scriptLoading ?? "defer";
		const configInject = c.inject ?? true;
		const inject =
			configInject === true
				? scriptLoading === "blocking"
					? "body"
					: "head"
				: configInject === false
					? "false"
					: configInject;
		return {
			...c,
			meta,
			scriptLoading,
			inject
		};
	}
);
