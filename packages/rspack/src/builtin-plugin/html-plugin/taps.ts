import * as binding from "@rspack/binding";
import type { Compiler } from "../../Compiler";
import { HtmlRspackPlugin } from "./plugin";

type HtmlPluginRegisterJsTapKeys = `registerHtmlPlugin${string}Taps`;
type HtmlPluginRegisterTapKeys<T> = T extends keyof binding.RegisterJsTaps
	? T extends HtmlPluginRegisterJsTapKeys
		? T
		: never
	: never;
type HtmlPluginTaps = {
	[K in HtmlPluginRegisterTapKeys<
		keyof binding.RegisterJsTaps
	>]: binding.RegisterJsTaps[K];
};

export function createHtmlPluginHooksRegisters(
	getCompiler: () => Compiler,
	createTap: Compiler["__internal__create_hook_register_taps"],
	createMapTap: Compiler["__internal__create_hook_map_register_taps"]
): HtmlPluginTaps {
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
}
