import type * as binding from "@rspack/binding";
import { AsyncSeriesBailHook, HookMap } from "tapable";

import { ResolveData, ResourceDataWithData } from "./Module";
import * as liteTapable from "./lite-tapable";

export type NormalModuleCreateData =
	binding.JsNormalModuleFactoryCreateModuleArgs & {
		settings: {};
	};

export class NormalModuleFactory {
	hooks: {
		// TODO: second param resolveData
		resolveForScheme: liteTapable.HookMap<
			liteTapable.AsyncSeriesBailHook<[ResourceDataWithData], true | void>
		>;
		beforeResolve: liteTapable.AsyncSeriesBailHook<[ResolveData], false | void>;
		afterResolve: liteTapable.AsyncSeriesBailHook<[ResolveData], false | void>;
		createModule: liteTapable.AsyncSeriesBailHook<
			[NormalModuleCreateData, {}],
			void
		>;
	};
	constructor() {
		this.hooks = {
			// /** @type {AsyncSeriesBailHook<[ResolveData], Module | false | void>} */
			// resolve: new AsyncSeriesBailHook(["resolveData"]),
			// /** @type {HookMap<AsyncSeriesBailHook<[ResourceDataWithData, ResolveData], true | void>>} */
			resolveForScheme: new liteTapable.HookMap(
				() => new liteTapable.AsyncSeriesBailHook(["resourceData"])
			),
			// /** @type {HookMap<AsyncSeriesBailHook<[ResourceDataWithData, ResolveData], true | void>>} */
			// resolveInScheme: new HookMap(
			// 	() => new AsyncSeriesBailHook(["resourceData", "resolveData"])
			// ),
			// /** @type {AsyncSeriesBailHook<[ResolveData], Module>} */
			// factorize: new AsyncSeriesBailHook(["resolveData"]),
			// /** @type {AsyncSeriesBailHook<[ResolveData], false | void>} */
			beforeResolve: new liteTapable.AsyncSeriesBailHook(["resolveData"]),
			// /** @type {AsyncSeriesBailHook<[ResolveData], false | void>} */
			afterResolve: new liteTapable.AsyncSeriesBailHook(["resolveData"]),
			// /** @type {AsyncSeriesBailHook<[ResolveData["createData"], ResolveData], Module | void>} */
			createModule: new liteTapable.AsyncSeriesBailHook([
				"createData",
				"resolveData"
			])
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
