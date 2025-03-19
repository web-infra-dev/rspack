import type {
	JsAfterEmitData,
	JsAfterTemplateExecutionData,
	JsAlterAssetTagGroupsData,
	JsAlterAssetTagsData,
	JsBeforeAssetTagGenerationData,
	JsBeforeEmitData
} from "@rspack/binding";
import * as liteTapable from "@rspack/lite-tapable";
import { Compilation } from "../../Compilation";
import type { HtmlRspackPluginOptions } from "./options";

const compilationHooksMap: WeakMap<Compilation, HtmlRspackPluginHooks> =
	new WeakMap();

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

export const getPluginHooks = (compilation: Compilation) => {
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

export const cleanPluginHooks = (compilation: Compilation) => {
	compilationHooksMap.delete(compilation);
};
