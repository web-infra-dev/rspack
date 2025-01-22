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
					return await queried.promise({
						...data,
						plugin: {
							options:
								HtmlRspackPlugin.getCompilationOptions(
									getCompiler().__internal__get_compilation()!
								) || {}
						}
					});
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
					return await queried.promise(data);
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
					return await queried.promise({
						...data,
						plugin: {
							options:
								HtmlRspackPlugin.getCompilationOptions(
									getCompiler().__internal__get_compilation()!
								) || {}
						}
					});
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
					return await queried.promise({
						...data,
						plugin: {
							options:
								HtmlRspackPlugin.getCompilationOptions(
									getCompiler().__internal__get_compilation()!
								) || {}
						}
					});
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
					return await queried.promise({
						...data,
						plugin: {
							options:
								HtmlRspackPlugin.getCompilationOptions(
									getCompiler().__internal__get_compilation()!
								) || {}
						}
					});
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
					return await queried.promise({
						...data,
						plugin: {
							options:
								HtmlRspackPlugin.getCompilationOptions(
									getCompiler().__internal__get_compilation()!
								) || {}
						}
					});
				};
			}
		)
	};
};
