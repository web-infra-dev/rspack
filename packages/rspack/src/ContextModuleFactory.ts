import type { JsContextModuleOptions } from "@rspack/binding";
import * as liteTapable from "@rspack/lite-tapable";
import type {
	ContextModuleFactoryAfterResolveResult,
	ContextModuleFactoryAlternativeRequests,
	ContextModuleFactoryBeforeResolveResult
} from "./Module";

export class ContextModuleFactory {
	hooks: {
		beforeResolve: liteTapable.AsyncSeriesWaterfallHook<
			[ContextModuleFactoryBeforeResolveResult],
			ContextModuleFactoryBeforeResolveResult | void
		>;
		afterResolve: liteTapable.AsyncSeriesWaterfallHook<
			[ContextModuleFactoryAfterResolveResult],
			ContextModuleFactoryAfterResolveResult | void
		>;
		alternativeRequests: liteTapable.AsyncSeriesWaterfallHook<
			[ContextModuleFactoryAlternativeRequests, JsContextModuleOptions],
			ContextModuleFactoryAlternativeRequests | void
		>;
	};
	constructor() {
		this.hooks = {
			beforeResolve: new liteTapable.AsyncSeriesWaterfallHook(["resolveData"]),
			afterResolve: new liteTapable.AsyncSeriesWaterfallHook(["resolveData"]),
			alternativeRequests: new liteTapable.AsyncSeriesWaterfallHook([
				"requests",
				"options"
			])
		};
	}
}
