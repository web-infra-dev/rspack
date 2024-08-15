import {
	BuiltinPluginName,
	type RawHtmlRspackPluginOptions
} from "@rspack/binding";
import { z } from "zod";

import path from "node:path";
import vm from "node:vm";
import type { Compiler } from "../Compiler";
import { Compilation, EntryPlugin, LoaderTargetPlugin } from "../exports";
import NodeTemplatePlugin from "../node/NodeTemplatePlugin";
import { validate } from "../util/validate";
import { EnableLibraryPlugin } from "./EnableLibraryPlugin";
import { NodeTargetPlugin } from "./NodeTargetPlugin";
import { create } from "./base";

const templateParameterFunc = z
	.function()
	.args(z.any())
	.returns(z.record(z.string()).or(z.promise(z.record(z.string()))));

const htmlRspackPluginOptions = z.strictObject({
	filename: z.string().optional(),
	template: z.string().optional(),
	templateContent: z.string().optional(),
	templateParameters: z.record(z.string()).or(templateParameterFunc).optional(),
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
export const HtmlRspackPlugin = create(
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
		let processType: "eval" | "compile" | "none" = "none";
		if (c.templateContent === undefined && c.template !== undefined) {
			if (c.template.includes("!")) {
				processType = "compile";
			} else {
				const extension = path.extname(c.template.split("?")[0]);
				if (extension === ".html" || extension === ".htm") {
					processType = "none";
				} else if (extension === ".js" || extension === ".cjs") {
					processType = "eval";
				} else {
					processType = "compile";
				}
			}
		}

		let internalTemplateCompileFn:
			| RawHtmlRspackPluginOptions["internalTemplateCompileFn"]
			| undefined = undefined;

		if (processType === "compile") {
			let compilation: Compilation | null = null;
			this.hooks.compilation.tap("HtmlRspackPlugin", c => {
				compilation = c;
			});
			internalTemplateCompileFn = async (request: string[]) => {
				if (!compilation) {
					throw new Error("no compilation");
				}
				const names: Record<string, string> = {};
				const childCompiler = compilation.createChildCompiler(
					"HtmlRspackCompiler",
					{
						filename: "__child-[name]",
						publicPath: "",
						library: {
							type: "var",
							name: "HTML_WEBPACK_PLUGIN_RESULT"
						},
						scriptType: /** @type {'text/javascript'} */ "text/javascript",
						iife: true
					},
					[
						new NodeTargetPlugin(),
						new NodeTemplatePlugin(),
						new LoaderTargetPlugin("node"),
						new EnableLibraryPlugin("var"),
						...request.map((r, idx) => {
							const name = `HtmlWebpackPlugin_${idx}`;
							names[r] = name;
							return new EntryPlugin(this.context, r, { name });
						})
					]
				);
				childCompiler.hooks.thisCompilation.tap(
					"HtmlRspackCompiler",
					compilation => {
						compilation.hooks.processAssets.tap(
							{
								name: "HtmlWebpackPlugin",
								stage: Compilation.PROCESS_ASSETS_STAGE_ADDITIONS
							},
							assets => {
								console.log(assets);
							}
						);
					}
				);

				return new Promise((resolve, reject) => {
					childCompiler?.runAsChild((err, entries, childCompilation) => {
						if (err) {
							return reject(err);
						}
						const res: Record<string, string> = {};

						Promise.all(
							Object.entries(names).map(async ([request, name]) => {
								const filename = `__child-${name}`;
								const asset = childCompilation?.getAsset(filename);
								if (asset) {
									const vmScript = new vm.Script(
										`${asset.source.source() as unknown as string};HTML_WEBPACK_PLUGIN_RESULT`,
										{
											filename: asset.name
										}
									);
									try {
										const generator = vmScript.runInThisContext();
										if (typeof generator === "string") {
											res[request] = generator;
										} else if (typeof generator === "function") {
											const params = {
												compilation,
												webpackConfig: this.options,
												htmlWebpackPlugin: {}
											};
											const finalParams =
												typeof c.templateParameters === "function"
													? await c.templateParameters(params)
													: {
															...params,
															...c.templateParameters
														};
											res[request] = generator(finalParams);
										}
									} catch (e) {
										console.error(e);
										return Promise.reject(e);
									}
								} else {
									// TODO: throw error
								}
							})
						).then(() => {
							resolve(res);
						});
					});
				});
			};
		} else if (processType === "eval") {
			let compilation: Compilation | null = null;
			this.hooks.compilation.tap("HtmlRspackPlugin", c => {
				compilation = c;
			});
			internalTemplateCompileFn = async (request: string[]) => {
				const res: Record<string, string> = {};

				await Promise.all(
					request.map(async r => {
						try {
							const generator = require(r);
							const params = {
								compilation,
								webpackConfig: this.options,
								htmlWebpackPlugin: {}
							};
							res[r] = generator(
								typeof c.templateParameters === "function"
									? await c.templateParameters(params)
									: {
											...params,
											...c.templateParameters
										}
							);
						} catch (e) {
							// TODO: handler error
						}
					})
				);

				return res;
			};
		} else {
			if (typeof c.templateParameters === "function") {
				throw new Error("function templateParameters is not supported");
			}
		}

		return {
			...c,
			meta,
			scriptLoading,
			inject,
			base,
			internalTemplateCompileFn,
			templateParameters:
				processType === "none"
					? (c.templateParameters as Record<string, string>)
					: {}
		};
	}
);
