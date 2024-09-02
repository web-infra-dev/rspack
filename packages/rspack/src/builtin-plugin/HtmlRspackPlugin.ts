import fs from "node:fs";
import path from "node:path";
import {
	BuiltinPluginName,
	type JsAfterEmitData,
	type JsAfterTemplateExecutionData,
	type JsAlterAssetTagGroupsData,
	type JsAlterAssetTagsData,
	type JsBeforeAssetTagGenerationData,
	type JsBeforeEmitData,
	type JsHtmlPluginTag,
	type RawHtmlRspackPluginOptions
} from "@rspack/binding";
import * as liteTapable from "@rspack/lite-tapable";
import { z } from "zod";

import { Compilation } from "../Compilation";
import type { Compiler } from "../Compiler";
import { validate } from "../util/validate";
import { create } from "./base";

type HtmlPluginTag = {
	tagName: string;
	attributes: Record<string, string>;
	voidTag: boolean;
	innerHTML?: string;
	toString?: () => string;
};

export type TemplateRenderFunction = (
	params: Record<string, any>
) => string | Promise<string>;

const templateRenderFunction = z
	.function()
	.args(z.record(z.string(), z.any()))
	.returns(
		z.string().or(z.promise(z.string()))
	) satisfies z.ZodType<TemplateRenderFunction>;

export type TemplateParamFunction = (
	params: Record<string, any>
) => Record<string, any> | Promise<Record<string, any>>;

const templateParamFunction = z
	.function()
	.args(z.record(z.string(), z.any()))
	.returns(
		z.record(z.string(), z.any()).or(z.promise(z.record(z.string(), z.any())))
	) satisfies z.ZodType<TemplateParamFunction>;

export type HtmlRspackPluginOptions = {
	/** The title to use for the generated HTML document. */
	title?: string;

	/**
	 * The file to write the HTML to. You can specify a subdirectory here too (eg: pages/index.html).
	 * @default 'index.html'
	 */
	filename?: string;

	/** The template file path. */
	template?: string;

	/**
	 * The template file content, priority is greater than template.
	 * When using a function, pass in the template parameters and use the returned string as the template content.
	 */
	templateContent?: string | TemplateRenderFunction;

	/**
	 * Allows to overwrite the parameters used in the template.
	 * When using a function, pass in the original template parameters and use the returned object as the final template parameters.
	 */
	templateParameters?: Record<string, string> | boolean | TemplateParamFunction;

	/**
	 * The script and link tag inject position in template. Use false to not inject.
	 * If not specified, it will be automatically determined based on scriptLoading.
	 */
	inject?: boolean | "head" | "body";

	/** The publicPath used for script and link tags. */
	publicPath?: string;

	/** Inject a [base](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base) tag */
	base?:
		| string
		| { href?: string; target?: "_self" | "_blank" | "_parent" | "_top" };

	/**
	 * Modern browsers support non blocking javascript loading ('defer') to improve the page startup performance.
	 * Setting to 'module' adds attribute type='module'.
	 * This also implies 'defer', since modules are automatically deferred.
	 * @default 'defer'
	 * */
	scriptLoading?: "blocking" | "defer" | "module" | "systemjs-module";

	/** Allows you to add only some chunks. */
	chunks?: string[];

	/** Allows you to skip some chunks. */
	excludeChunks?: string[];

	/** The sri hash algorithm, disabled by default. */
	sri?: "sha256" | "sha384" | "sha512";

	/**
	 * Controls whether to minify the output.
	 * @default false
	 */
	minify?: boolean;

	/** Adds the given favicon path to the output HTML. */
	favicon?: string;

	/**
	 * If true then append a unique rspack compilation hash to all included scripts and CSS files.
	 * This is useful for cache busting
	 * @default false
	 * */
	meta?: Record<string, string | Record<string, string>>;

	/** Inject a base tag */
	hash?: boolean;
};

const templateFilenameFunction = z
	.function()
	.args(z.string())
	.returns(z.string());

const htmlRspackPluginOptions = z.strictObject({
	filename: z.string().or(templateFilenameFunction).optional(),
	template: z
		.string()
		.refine(
			val => !val.includes("!"),
			() => ({
				message:
					"HtmlRspackPlugin does not support template path with loader yet"
			})
		)
		.optional(),
	templateContent: z.string().or(templateRenderFunction).optional(),
	templateParameters: z
		.record(z.string())
		.or(z.boolean())
		.or(templateParamFunction)
		.optional(),
	inject: z.enum(["head", "body"]).or(z.boolean()).optional(),
	publicPath: z.string().optional(),
	base: z
		.string()
		.or(
			z.strictObject({
				href: z.string().optional(),
				target: z.enum(["_self", "_blank", "_parent", "_top"]).optional()
			})
		)
		.optional(),
	scriptLoading: z
		.enum(["blocking", "defer", "module", "systemjs-module"])
		.optional(),
	chunks: z.string().array().optional(),
	excludeChunks: z.string().array().optional(),
	sri: z.enum(["sha256", "sha384", "sha512"]).optional(),
	minify: z.boolean().optional(),
	title: z.string().optional(),
	favicon: z.string().optional(),
	meta: z.record(z.string().or(z.record(z.string()))).optional(),
	hash: z.boolean().optional()
}) satisfies z.ZodType<HtmlRspackPluginOptions>;

const HtmlRspackPluginImpl = create(
	BuiltinPluginName.HtmlRspackPlugin,
	function (
		this: Compiler,
		c: HtmlRspackPluginOptions = {}
	): RawHtmlRspackPluginOptions {
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
		const base = typeof c.base === "string" ? { href: c.base } : c.base;

		let compilation: Compilation | null = null;
		this.hooks.compilation.tap("HtmlRspackPlugin", compilationInstance => {
			compilation = compilationInstance;
			compilationOptionsMap.set(compilation, c);
		});
		this.hooks.done.tap("HtmlRspackPlugin", stats => {
			compilationHooksMap.delete(stats.compilation);
			compilationOptionsMap.delete(stats.compilation);
		});

		function generateRenderData(data: string): Record<string, unknown> {
			const json = JSON.parse(data);
			if (typeof c.templateParameters !== "function") {
				json.compilation = compilation;
			}
			const renderTag = function (this: HtmlPluginTag) {
				return htmlTagObjectToString(this);
			};
			const renderTagList = function (this: HtmlPluginTag[]) {
				return this.join("");
			};
			if (Array.isArray(json.htmlRspackPlugin?.tags?.headTags)) {
				for (const tag of json.htmlRspackPlugin.tags.headTags) {
					tag.toString = renderTag;
				}
				json.htmlRspackPlugin.tags.headTags.toString = renderTagList;
			}
			if (Array.isArray(json.htmlRspackPlugin?.tags?.bodyTags)) {
				for (const tag of json.htmlRspackPlugin.tags.bodyTags) {
					tag.toString = renderTag;
				}
				json.htmlRspackPlugin.tags.bodyTags.toString = renderTagList;
			}
			return json;
		}

		let templateContent = c.templateContent;
		let templateFn = undefined;
		if (typeof templateContent === "function") {
			templateFn = async (data: string) => {
				try {
					const renderer = c.templateContent as (
						data: Record<string, unknown>
					) => Promise<string> | string;
					if (c.templateParameters === false) {
						return await renderer({});
					}
					return await renderer(generateRenderData(data));
				} catch (e) {
					const error = new Error(
						`HtmlRspackPlugin: render template function failed, ${(e as Error).message}`
					);
					error.stack = (e as Error).stack;
					throw error;
				}
			};
			templateContent = "";
		} else if (c.template) {
			const filename = c.template.split("?")[0];
			if ([".js", ".cjs"].includes(path.extname(filename))) {
				templateFn = async (data: string) => {
					const context = this.options.context || process.cwd();
					const templateFilePath = path.resolve(context, filename);
					if (!fs.existsSync(templateFilePath)) {
						throw new Error(
							`HtmlRspackPlugin: could not load file \`${filename}\` from \`${context}\``
						);
					}
					try {
						const renderer = require(templateFilePath) as (
							data: Record<string, unknown>
						) => Promise<string> | string;
						if (c.templateParameters === false) {
							return await renderer({});
						}
						return await renderer(generateRenderData(data));
					} catch (e) {
						const error = new Error(
							`HtmlRspackPlugin: render template function failed, ${(e as Error).message}`
						);
						error.stack = (e as Error).stack;
						throw error;
					}
				};
			}
		}

		const rawTemplateParameters = c.templateParameters;
		let templateParameters:
			| boolean
			| Record<string, any>
			| ((params: string) => Promise<string>)
			| undefined;
		if (typeof rawTemplateParameters === "function") {
			templateParameters = async (data: string) => {
				const newData = await rawTemplateParameters(JSON.parse(data));
				return JSON.stringify(newData);
			};
		} else {
			templateParameters = rawTemplateParameters;
		}

		let filenames: Set<string> | undefined = undefined;
		if (typeof c.filename === "string") {
			filenames = new Set();
			if (c.filename.includes("[name]")) {
				if (typeof this.options.entry === "object") {
					for (const entryName of Object.keys(this.options.entry)) {
						filenames.add(c.filename.replace(/\[name\]/g, entryName));
					}
				} else {
					throw new Error(
						"HtmlRspackPlugin: filename with `[name]` does not support function entry"
					);
				}
			} else {
				filenames.add(c.filename);
			}
		} else if (typeof c.filename === "function") {
			filenames = new Set();
			if (typeof this.options.entry === "object") {
				for (const entryName of Object.keys(this.options.entry)) {
					filenames.add(c.filename(entryName));
				}
			} else {
				throw new Error(
					"HtmlRspackPlugin: function filename does not support function entry"
				);
			}
		}

		return {
			filename: filenames ? Array.from(filenames) : undefined,
			template: c.template,
			hash: c.hash,
			title: c.title,
			favicon: c.favicon,
			publicPath: c.publicPath,
			chunks: c.chunks,
			excludeChunks: c.excludeChunks,
			sri: c.sri,
			minify: c.minify,
			meta,
			scriptLoading,
			inject,
			base,
			templateFn,
			templateContent,
			templateParameters
		};
	}
);

function htmlTagObjectToString(tag: {
	tagName: string;
	attributes: Record<string, string>;
	voidTag: boolean;
	innerHTML?: string;
}) {
	const attributes = Object.keys(tag.attributes || {})
		.filter(
			attributeName =>
				tag.attributes[attributeName] === "" || tag.attributes[attributeName]
		)
		.map(attributeName => {
			if (tag.attributes[attributeName] === "true") {
				return attributeName;
			}
			return `${attributeName}="${tag.attributes[attributeName]}"`;
		});
	const res = `<${[tag.tagName].concat(attributes).join(" ")}${tag.voidTag && !tag.innerHTML ? "/" : ""}>${tag.innerHTML || ""}${tag.voidTag && !tag.innerHTML ? "" : `</${tag.tagName}>`}`;
	return res;
}

type ExtraPluginHookData = {
	plugin: {
		options: HtmlRspackPluginOptions;
	};
};

export type HtmlRspackPluginHooks = {
	beforeAssetTagGeneration: liteTapable.AsyncSeriesWaterfallHook<
		[JsBeforeAssetTagGenerationData & ExtraPluginHookData]
	>;
	alterAssetTags: liteTapable.AsyncSeriesWaterfallHook<[JsAlterAssetTagsData]>;
	alterAssetTagGroups: liteTapable.AsyncSeriesWaterfallHook<
		[JsAlterAssetTagGroupsData & ExtraPluginHookData]
	>;
	afterTemplateExecution: liteTapable.AsyncSeriesWaterfallHook<
		[JsAfterTemplateExecutionData & ExtraPluginHookData]
	>;
	beforeEmit: liteTapable.AsyncSeriesWaterfallHook<
		[JsBeforeEmitData & ExtraPluginHookData]
	>;
	afterEmit: liteTapable.AsyncSeriesWaterfallHook<
		[JsAfterEmitData & ExtraPluginHookData]
	>;
};

const compilationHooksMap: WeakMap<Compilation, HtmlRspackPluginHooks> =
	new WeakMap();

const compilationOptionsMap: WeakMap<Compilation, HtmlRspackPluginOptions> =
	new WeakMap();

const HtmlRspackPlugin = HtmlRspackPluginImpl as typeof HtmlRspackPluginImpl & {
	/**
	 * @deprecated Use `getCompilationHooks` instead.
	 */
	getHooks: (compilation: Compilation) => HtmlRspackPluginHooks;
	getCompilationHooks: (compilation: Compilation) => HtmlRspackPluginHooks;
	getCompilationOptions: (
		compilation: Compilation
	) => HtmlRspackPluginOptions | void;
	createHtmlTagObject: (
		tagName: string,
		attributes?: Record<string, string | boolean>,
		innerHTML?: string | undefined
	) => JsHtmlPluginTag;
	version: number;
};

const voidTags = [
	"area",
	"base",
	"br",
	"col",
	"embed",
	"hr",
	"img",
	"input",
	"keygen",
	"link",
	"meta",
	"param",
	"source",
	"track",
	"wbr"
];

HtmlRspackPlugin.createHtmlTagObject = (
	tagName: string,
	attributes?: Record<string, string | boolean>,
	innerHTML?: string | undefined
): JsHtmlPluginTag => {
	return {
		tagName,
		voidTag: voidTags.includes(tagName),
		attributes: attributes || {},
		innerHTML
	};
};

HtmlRspackPlugin.getCompilationOptions = (compilation: Compilation) => {
	if (!(compilation instanceof Compilation)) {
		throw new TypeError(
			"The 'compilation' argument must be an instance of Compilation"
		);
	}
	return compilationOptionsMap.get(compilation);
};

HtmlRspackPlugin.getHooks = HtmlRspackPlugin.getCompilationHooks = (
	compilation: Compilation
) => {
	if (!(compilation instanceof Compilation)) {
		throw new TypeError(
			"The 'compilation' argument must be an instance of Compilation"
		);
	}
	let hooks = compilationHooksMap.get(compilation);
	if (hooks === undefined) {
		hooks = {
			beforeAssetTagGeneration: new liteTapable.AsyncSeriesWaterfallHook([
				"data"
			]),
			alterAssetTags: new liteTapable.AsyncSeriesWaterfallHook(["data"]),
			alterAssetTagGroups: new liteTapable.AsyncSeriesWaterfallHook(["data"]),
			afterTemplateExecution: new liteTapable.AsyncSeriesWaterfallHook([
				"data"
			]),
			beforeEmit: new liteTapable.AsyncSeriesWaterfallHook(["data"]),
			afterEmit: new liteTapable.AsyncSeriesWaterfallHook(["data"])
		};
		compilationHooksMap.set(compilation, hooks);
	}
	return hooks;
};

HtmlRspackPlugin.version = 5;

export { HtmlRspackPlugin };
