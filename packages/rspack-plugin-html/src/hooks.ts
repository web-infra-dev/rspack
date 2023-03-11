/**
 * The following code is modified based on
 * https://github.com/jantimon/html-webpack-plugin/blob/d5ce5a8f2d12a2450a65ec51c285dd54e36cd921/lib/hooks.js
 *
 * MIT Licensed
 * Author Jan Nicklas
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/jantimon/html-webpack-plugin/blob/d5ce5a8f2d12a2450a65ec51c285dd54e36cd921/LICENSE
 */
import { Compilation } from "@rspack/core";
import { AsyncSeriesWaterfallHook } from "tapable";
import HTMLRspackPlugin, { HtmlTagObject } from "./index";

const htmlRspackPluginHooksMap = new WeakMap<
	Compilation,
	HtmlRspackPluginHooks
>();

export type HtmlRspackPluginHooks = ReturnType<
	typeof createHtmlRspackPluginHooks
>;

export function getHtmlRspackPluginHooks(compilation: Compilation) {
	let hooks = htmlRspackPluginHooksMap.get(compilation);
	// Setup the hooks only once
	if (hooks === undefined) {
		hooks = createHtmlRspackPluginHooks();
		htmlRspackPluginHooksMap.set(compilation, hooks);
	}
	return hooks;
}

function createHtmlRspackPluginHooks() {
	return {
		beforeAssetTagGeneration: new AsyncSeriesWaterfallHook<{
			assets: {
				publicPath: string;
				js: string[];
				css: string[];
				favicon?: string | undefined;
				manifest?: string | undefined;
			};
			outputName: string;
			plugin: HTMLRspackPlugin;
		}>(["pluginArgs"]),
		alterAssetTags: new AsyncSeriesWaterfallHook<{
			assetTags: {
				scripts: HtmlTagObject[];
				styles: HtmlTagObject[];
				meta: HtmlTagObject[];
			};
			publicPath: string;
			outputName: string;
			plugin: HTMLRspackPlugin;
		}>(["pluginArgs"]),
		alterAssetTagGroups: new AsyncSeriesWaterfallHook<{
			headTags: HtmlTagObject[];
			bodyTags: HtmlTagObject[];
			publicPath: string;
			outputName: string;
			plugin: HTMLRspackPlugin;
		}>(["pluginArgs"]),
		afterTemplateExecution: new AsyncSeriesWaterfallHook<{
			html: string;
			headTags: HtmlTagObject[];
			bodyTags: HtmlTagObject[];
			outputName: string;
			plugin: HTMLRspackPlugin;
		}>(["pluginArgs"]),
		beforeEmit: new AsyncSeriesWaterfallHook<{
			html: string;
			outputName: string;
			plugin: HTMLRspackPlugin;
		}>(["pluginArgs"]),
		afterEmit: new AsyncSeriesWaterfallHook<{
			outputName: string;
			plugin: HTMLRspackPlugin;
		}>(["pluginArgs"])
	};
}
