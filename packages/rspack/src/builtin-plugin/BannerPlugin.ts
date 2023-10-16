import { z } from "zod";
import {
	JsChunk,
	RawBannerContent,
	RawBannerPluginOptions,
	RawBannerRule,
	RawBannerRules
} from "@rspack/binding";
import { BuiltinPluginName, create } from "./base";

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
	test: rules.optional()
});
export type BannerPluginOptions = z.infer<typeof bannerPluginOptions>;

const bannerPluginArgument = bannerContent.or(bannerPluginOptions);
export type BannerPluginArgument = z.infer<typeof bannerPluginArgument>;

function getRawBannerRule(condition: Rule): RawBannerRule {
	if (typeof condition === "string") {
		return {
			type: "string",
			stringMatcher: condition
		};
	}
	if (condition instanceof RegExp) {
		return {
			type: "regexp",
			regexpMatcher: condition.source
		};
	}
	throw new Error("unreachable: condition should be one of string, RegExp");
}

function getRawBannerRules(condition?: Rules): RawBannerRules | undefined {
	if (!condition) return undefined;

	if (Array.isArray(condition)) {
		return {
			type: "array",
			arrayMatcher: condition.map(i => getRawBannerRule(i))
		};
	}

	return getRawBannerRule(condition);
}

function getRawBannerContent(content: BannerContent): RawBannerContent {
	if (typeof content === "string") {
		return {
			type: "string",
			stringPayload: content
		};
	}
	if (typeof content === "function") {
		return {
			type: "function",
			fnPayload: content
		};
	}
	throw new Error("BannerContent should be a string or function");
}

export const BannerPlugin = create(
	BuiltinPluginName.BannerPlugin,
	(args: BannerPluginArgument): RawBannerPluginOptions => {
		if (typeof args === "string") {
			return {
				banner: getRawBannerContent(args)
			};
		}
		if (typeof args === "function") {
			return {
				banner: getRawBannerContent(args)
			};
		}

		return {
			banner: getRawBannerContent(args.banner),
			entryOnly: args.entryOnly,
			footer: args.footer,
			raw: args.raw,
			test: getRawBannerRules(args.test),
			include: getRawBannerRules(args.include),
			exclude: getRawBannerRules(args.exclude)
		};
	}
);
