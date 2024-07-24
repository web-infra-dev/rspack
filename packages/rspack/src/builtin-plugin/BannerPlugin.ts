import {
	BuiltinPluginName,
	type JsChunk,
	type RawBannerPluginOptions
} from "@rspack/binding";
import { z } from "zod";

import { create } from "./base";

const rule = z.string().or(z.instanceof(RegExp));
export type Rule = z.infer<typeof rule>;

const rules = rule.or(rule.array());
export type Rules = z.infer<typeof rules>;

const bannerFunction = z
	.function()
	.args(
		z.object({
			hash: z.string(),
			chunk: z.custom<JsChunk>(),
			filename: z.string()
		})
	)
	.returns(z.string());
export type BannerFunction = z.infer<typeof bannerFunction>;

const bannerContent = z.string().or(bannerFunction);
export type BannerContent = z.infer<typeof bannerContent>;

const bannerPluginOptions = z.strictObject({
	banner: bannerContent,
	entryOnly: z.boolean().optional(),
	exclude: rules.optional(),
	include: rules.optional(),
	raw: z.boolean().optional(),
	footer: z.boolean().optional(),
	stage: z.number().optional(),
	test: rules.optional()
});
export type BannerPluginOptions = z.infer<typeof bannerPluginOptions>;

const bannerPluginArgument = bannerContent.or(bannerPluginOptions);
export type BannerPluginArgument = z.infer<typeof bannerPluginArgument>;

export const BannerPlugin = create(
	BuiltinPluginName.BannerPlugin,
	(args: BannerPluginArgument): RawBannerPluginOptions => {
		if (typeof args === "string" || typeof args === "function") {
			return {
				banner: args
			};
		}

		return {
			banner: args.banner,
			entryOnly: args.entryOnly,
			footer: args.footer,
			raw: args.raw,
			test: args.test,
			stage: args.stage,
			include: args.include,
			exclude: args.exclude
		};
	}
);
