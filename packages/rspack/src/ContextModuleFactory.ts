import * as liteTapable from "@rspack/lite-tapable";
import type {
	ContextModuleFactoryAfterResolveResult,
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
	};
	constructor() {
		this.hooks = {
			beforeResolve: new liteTapable.AsyncSeriesWaterfallHook(["resolveData"]),
			afterResolve: new liteTapable.AsyncSeriesWaterfallHook(["resolveData"])
		};
	}
}
