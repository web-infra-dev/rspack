import * as liteTapable from "./lite-tapable";
import {
	ContextModuleFactoryAfterResolveResult,
	ContextModuleFactoryBeforeResolveResult
} from "./Module";

export class ContextModuleFactory {
	hooks: {
		// TODO: second param resolveData
		// resolveForScheme: HookMap<
		// 	AsyncSeriesBailHook<[ResourceDataWithData], true | void>
		// >;
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
			// /** @type {AsyncSeriesBailHook<[ResolveData], Module | false | void>} */
			// resolve: new AsyncSeriesBailHook(["resolveData"]),
			// /** @type {HookMap<AsyncSeriesBailHook<[ResourceDataWithData, ResolveData], true | void>>} */
			// resolveForScheme: new HookMap(
			// 	() => new AsyncSeriesBailHook(["resourceData"])
			// ),
			// /** @type {HookMap<AsyncSeriesBailHook<[ResourceDataWithData, ResolveData], true | void>>} */
			// resolveInScheme: new HookMap(
			// 	() => new AsyncSeriesBailHook(["resourceData", "resolveData"])
			// ),
			// /** @type {AsyncSeriesBailHook<[ResolveData], Module>} */
			// factorize: new AsyncSeriesBailHook(["resolveData"]),
			// /** @type {AsyncSeriesBailHook<[ResolveData], false | void>} */
			beforeResolve: new liteTapable.AsyncSeriesWaterfallHook(["resolveData"]),
			afterResolve: new liteTapable.AsyncSeriesWaterfallHook(["resolveData"])
			// /** @type {AsyncSeriesBailHook<[ResolveData["createData"], ResolveData], Module | void>} */
			// createModule: new AsyncSeriesBailHook(["createData", "resolveData"]),
			// /** @type {SyncWaterfallHook<[Module, ResolveData["createData"], ResolveData], Module>} */
			// module: new SyncWaterfallHook(["module", "createData", "resolveData"]),
			// createParser: new HookMap(() => new SyncBailHook(["parserOptions"])),
			// parser: new HookMap(() => new SyncHook(["parser", "parserOptions"])),
			// createGenerator: new HookMap(
			// 	() => new SyncBailHook(["generatorOptions"])
			// ),
			// generator: new HookMap(
			// 	() => new SyncHook(["generator", "generatorOptions"])
			// )
		};
	}
}
