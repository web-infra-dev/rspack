import type { JsBuildMeta } from "@rspack/binding";
import * as z from "zod/v4";
import type {
	HtmlRspackPluginOptions,
	TemplateParamFunction,
	TemplateRenderFunction
} from "../builtin-plugin/html-plugin";
import type { IgnorePluginOptions } from "../builtin-plugin/IgnorePlugin";
import type { RsdoctorPluginOptions } from "../builtin-plugin/RsdoctorPlugin";
import type { SubresourceIntegrityPluginOptions } from "../builtin-plugin/SubresourceIntegrityPlugin";
import type { DllPluginOptions } from "../lib/DllPlugin";
import type {
	DllReferencePluginOptions,
	DllReferencePluginOptionsContent,
	DllReferencePluginOptionsManifest,
	DllReferencePluginOptionsSourceType
} from "../lib/DllReferencePlugin";
import { memoize } from "../util/memoize";
import { anyFunction, numberOrInfinity } from "./utils";

export const getIgnorePluginOptionsSchema = memoize(
	() =>
		z.union([
			z.object({
				contextRegExp: z.instanceof(RegExp).optional(),
				resourceRegExp: z.instanceof(RegExp)
			}),
			z.object({
				checkResource: anyFunction
			})
		]) satisfies z.ZodType<IgnorePluginOptions>
);

export const getRsdoctorPluginSchema = memoize(
	() =>
		z.strictObject({
			moduleGraphFeatures: z
				.union([z.boolean(), z.array(z.enum(["graph", "ids", "sources"]))])
				.optional(),
			chunkGraphFeatures: z
				.union([z.boolean(), z.array(z.enum(["graph", "assets"]))])
				.optional(),
			sourceMapFeatures: z
				.object({
					module: z.boolean().optional(),
					cheap: z.boolean().optional()
				})
				.optional()
		}) satisfies z.ZodType<RsdoctorPluginOptions>
);

export const getSRIPluginOptionsSchema = memoize(() => {
	const hashFunctionSchema = z.enum(["sha256", "sha384", "sha512"]);

	return z.object({
		hashFuncNames: z
			.tuple([hashFunctionSchema])
			.rest(hashFunctionSchema)
			.optional(),
		htmlPlugin: z.string().or(z.literal(false)).optional(),
		enabled: z.literal("auto").or(z.boolean()).optional()
	}) satisfies z.ZodType<SubresourceIntegrityPluginOptions>;
});

export const getDllPluginOptionsSchema = memoize(
	() =>
		z.object({
			context: z.string().optional(),
			entryOnly: z.boolean().optional(),
			format: z.boolean().optional(),
			name: z.string().optional(),
			path: z.string(),
			type: z.string().optional()
		}) satisfies z.ZodType<DllPluginOptions>
);

export const getDllReferencePluginOptionsSchema = memoize(() => {
	const dllReferencePluginOptionsContentItem = z
		.object({
			buildMeta: z.custom<JsBuildMeta>(),
			exports: z.array(z.string()).or(z.literal(true)),
			id: z.string().or(numberOrInfinity)
		})
		.partial();

	const dllReferencePluginOptionsContent = z.record(
		z.string(),
		dllReferencePluginOptionsContentItem
	) satisfies z.ZodType<DllReferencePluginOptionsContent>;

	const dllReferencePluginOptionsSourceType = z.enum([
		"var",
		"assign",
		"this",
		"window",
		"global",
		"commonjs",
		"commonjs2",
		"commonjs-module",
		"amd",
		"amd-require",
		"umd",
		"umd2",
		"jsonp",
		"system"
	]) satisfies z.ZodType<DllReferencePluginOptionsSourceType>;

	const dllReferencePluginOptionsManifest = z.object({
		content: dllReferencePluginOptionsContent,
		name: z.string().optional(),
		type: dllReferencePluginOptionsSourceType.optional()
	}) satisfies z.ZodType<DllReferencePluginOptionsManifest>;

	const dllReferencePluginOptions = z.union([
		z.object({
			context: z.string().optional(),
			extensions: z.array(z.string()).optional(),
			manifest: z.string().or(dllReferencePluginOptionsManifest),
			name: z.string().optional(),
			scope: z.string().optional(),
			sourceType: dllReferencePluginOptionsSourceType.optional(),
			type: z.enum(["require", "object"]).optional()
		}),
		z.object({
			content: dllReferencePluginOptionsContent,
			context: z.string().optional(),
			extensions: z.array(z.string()).optional(),
			name: z.string(),
			scope: z.string().optional(),
			sourceType: dllReferencePluginOptionsSourceType.optional(),
			type: z.enum(["require", "object"]).optional()
		})
	]) satisfies z.ZodType<DllReferencePluginOptions>;

	return dllReferencePluginOptions;
});

export const getHtmlPluginOptionsSchema = memoize(() => {
	const templateRenderFunction =
		anyFunction satisfies z.ZodType<TemplateRenderFunction>;
	const templateParamFunction =
		anyFunction satisfies z.ZodType<TemplateParamFunction>;

	return z
		.object({
			filename: z.string().or(anyFunction),
			template: z.string().refine(val => !val.includes("!"), {
				error: "HtmlRspackPlugin does not support template path with loader yet"
			}),
			templateContent: z.string().or(templateRenderFunction),
			templateParameters: z
				.record(z.string(), z.string())
				.or(z.boolean())
				.or(templateParamFunction),
			inject: z.enum(["head", "body"]).or(z.boolean()),
			publicPath: z.string(),
			base: z.string().or(
				z
					.strictObject({
						href: z.string(),
						target: z.enum(["_self", "_blank", "_parent", "_top"])
					})
					.partial()
			),
			scriptLoading: z.enum(["blocking", "defer", "module", "systemjs-module"]),
			chunks: z.string().array(),
			excludeChunks: z.string().array(),
			chunksSortMode: z.enum(["auto", "manual"]),
			sri: z.enum(["sha256", "sha384", "sha512"]),
			minify: z.boolean(),
			title: z.string(),
			favicon: z.string(),
			meta: z.record(
				z.string(),
				z.string().or(z.record(z.string(), z.string()))
			),
			hash: z.boolean()
		})
		.partial() satisfies z.ZodType<HtmlRspackPluginOptions>;
});
