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

const templateRenderFunction = z
	.function()
	.args(z.record(z.string(), z.any()))
	.returns(z.string().or(z.promise(z.string())));

const templateParamFunction = z
	.function()
	.args(z.record(z.string(), z.any()))
	.returns(
		z.record(z.string(), z.any()).or(z.promise(z.record(z.string(), z.any())))
	);

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
});
export type HtmlRspackPluginOptions = z.infer<typeof htmlRspackPluginOptions>;

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
		let templateParameters;
		if (typeof rawTemplateParameters === "function") {
			templateParameters = async (data: string) => {
				const newData = await rawTemplateParameters(JSON.parse(data));
				return JSON.stringify(newData);
			};
		} else {
			templateParameters = rawTemplateParameters;
		}

		const addedFilename: Set<string> = new Set();
		let filenames: string[] | undefined = undefined;
		if (typeof c.filename === "string") {
			filenames = [];
			if (c.filename.includes("[name]")) {
				if (typeof this.options.entry === "object") {
					for (const entryName of Object.keys(this.options.entry)) {
						const filename = c.filename.replace(/\[name\]/g, entryName);
						if (!addedFilename.has(filename)) {
							filenames.push(filename);
							addedFilename.add(filename);
						}
					}
				} else {
					throw new Error(
						"HtmlRspackPlugin: filename with `[name]` does not support function entry"
					);
				}
			} else {
				filenames.push(c.filename);
			}
		} else if (typeof c.filename === "function") {
			filenames = [];
			if (typeof this.options.entry === "object") {
				for (const entryName of Object.keys(this.options.entry)) {
					const filename = c.filename(entryName);
					if (!addedFilename.has(filename)) {
						filenames.push(filename);
						addedFilename.add(filename);
					}
				}
			} else {
				throw new Error(
					"HtmlRspackPlugin: function filename does not support function entry"
				);
			}
		}

		return {
			filename: filenames,
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
	getCompilationHooks: (compilation: Compilation) => HtmlRspackPluginHooks;
	getCompilationOptions: (
		compilation: Compilation
	) => HtmlRspackPluginOptions | void;
	createHtmlTagObject: (
		tagName: string,
		attributes?: Record<string, string | boolean>,
		innerHTML?: string | undefined
	) => JsHtmlPluginTag;
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

HtmlRspackPlugin.getCompilationHooks = (compilation: Compilation) => {
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

export { HtmlRspackPlugin };
