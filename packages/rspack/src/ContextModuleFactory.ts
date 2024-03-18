import { AsyncSeriesBailHook } from "tapable";

// type ResourceData = {
// 	resource: string;
// 	path: string;
// 	query?: string;
// 	fragment?: string;
// };
// resource: uri,
// resource_path: info.path,
// resource_query: (!info.query.is_empty()).then_some(info.query),
// resource_fragment: (!info.fragment.is_empty()).then_some(info.fragment),
// type ResourceDataWithData = ResourceData & { data?: Record<string, any> };
type ResolveData = {
	context?: string;
	request: string;
	// assertions: Record<string, any> | undefined;
	// dependencies: ModuleDependency[];
};

export class ContextModuleFactory {
	hooks: {
		// TODO: second param resolveData
		// resolveForScheme: HookMap<
		// 	AsyncSeriesBailHook<[ResourceDataWithData], true | void>
		// >;
		beforeResolve: AsyncSeriesBailHook<[ResolveData], boolean | void>;
		afterResolve: AsyncSeriesBailHook<[ResolveData], boolean | void>;
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
			beforeResolve: new AsyncSeriesBailHook(["resolveData"]),
			afterResolve: new AsyncSeriesBailHook(["resolveData"])
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
