import * as binding from "@rspack/binding";
import type { CreatePartialRegisters } from "../../taps/types";
import { HtmlRspackPlugin } from "./plugin";

export const createHtmlPluginHooksRegisters: CreatePartialRegisters<
	`HtmlPlugin`
> = (getCompiler, createTap, createMapTap) => {
	return {
		registerHtmlPluginBeforeAssetTagGenerationTaps: createTap(
			binding.RegisterJsTapKind.HtmlPluginBeforeAssetTagGeneration,
			function () {
				return HtmlRspackPlugin.getCompilationHooks(
					getCompiler().__internal__get_compilation()!
				).beforeAssetTagGeneration;
			},
			function (queried) {
				return async function (data: binding.JsBeforeAssetTagGenerationData) {
					const compilationId = data.compilationId;
					const res = await queried.promise({
						...data,
						plugin: {
							options:
								HtmlRspackPlugin.getCompilationOptions(
									getCompiler().__internal__get_compilation()!
								) || {}
						}
					});
					res.compilationId = compilationId;
					return res;
				};
			}
		),
		registerHtmlPluginAlterAssetTagsTaps: createTap(
			binding.RegisterJsTapKind.HtmlPluginAlterAssetTags,
			function () {
				return HtmlRspackPlugin.getCompilationHooks(
					getCompiler().__internal__get_compilation()!
				).alterAssetTags;
			},
			function (queried) {
				return async function (data: binding.JsAlterAssetTagsData) {
					const compilationId = data.compilationId;
					const res = await queried.promise(data);
					res.compilationId = compilationId;
					return res;
				};
			}
		),
		registerHtmlPluginAlterAssetTagGroupsTaps: createTap(
			binding.RegisterJsTapKind.HtmlPluginAlterAssetTagGroups,
			function () {
				return HtmlRspackPlugin.getCompilationHooks(
					getCompiler().__internal__get_compilation()!
				).alterAssetTagGroups;
			},
			function (queried) {
				return async function (data: binding.JsAlterAssetTagGroupsData) {
					const compilationId = data.compilationId;
					const res = await queried.promise({
						...data,
						plugin: {
							options:
								HtmlRspackPlugin.getCompilationOptions(
									getCompiler().__internal__get_compilation()!
								) || {}
						}
					});
					res.compilationId = compilationId;
					return res;
				};
			}
		),
		registerHtmlPluginAfterTemplateExecutionTaps: createTap(
			binding.RegisterJsTapKind.HtmlPluginAfterTemplateExecution,
			function () {
				return HtmlRspackPlugin.getCompilationHooks(
					getCompiler().__internal__get_compilation()!
				).afterTemplateExecution;
			},
			function (queried) {
				return async function (data: binding.JsAfterTemplateExecutionData) {
					const compilationId = data.compilationId;
					const res = await queried.promise({
						...data,
						plugin: {
							options:
								HtmlRspackPlugin.getCompilationOptions(
									getCompiler().__internal__get_compilation()!
								) || {}
						}
					});
					res.compilationId = compilationId;
					return res;
				};
			}
		),
		registerHtmlPluginBeforeEmitTaps: createTap(
			binding.RegisterJsTapKind.HtmlPluginBeforeEmit,
			function () {
				return HtmlRspackPlugin.getCompilationHooks(
					getCompiler().__internal__get_compilation()!
				).beforeEmit;
			},
			function (queried) {
				return async function (data: binding.JsBeforeEmitData) {
					const compilationId = data.compilationId;
					const res = await queried.promise({
						...data,
						plugin: {
							options:
								HtmlRspackPlugin.getCompilationOptions(
									getCompiler().__internal__get_compilation()!
								) || {}
						}
					});
					res.compilationId = compilationId;
					return res;
				};
			}
		),
		registerHtmlPluginAfterEmitTaps: createTap(
			binding.RegisterJsTapKind.HtmlPluginAfterEmit,
			function () {
				return HtmlRspackPlugin.getCompilationHooks(
					getCompiler().__internal__get_compilation()!
				).afterEmit;
			},
			function (queried) {
				return async function (data: binding.JsAfterEmitData) {
					const compilationId = data.compilationId;
					const res = await queried.promise({
						...data,
						plugin: {
							options:
								HtmlRspackPlugin.getCompilationOptions(
									getCompiler().__internal__get_compilation()!
								) || {}
						}
					});
					res.compilationId = compilationId;
					return res;
				};
			}
		)
	};
};
