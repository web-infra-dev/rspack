import { AsyncSeriesBailHook, HookMap } from "tapable";
import * as liteTapable from "./lite-tapable";
import type * as binding from "@rspack/binding";

type ResourceData = {
	resource: string;
	path: string;
	query?: string;
	fragment?: string;
};
// resource: uri,
// resource_path: info.path,
// resource_query: (!info.query.is_empty()).then_some(info.query),
// resource_fragment: (!info.fragment.is_empty()).then_some(info.fragment),
type ResourceDataWithData = ResourceData & { data?: Record<string, any> };
// type CreateData = Partial<NormalModuleCreateData & {settings: ModuleSettings}>;
type ResolveData = {
	context?: string;
	request: string;
	fileDependencies: string[];
	missingDependencies: string[];
	contextDependencies: string[];
	// createData: CreateData;
	// assertions: Record<string, any> | undefined;
	// dependencies: ModuleDependency[];
};

type CreateModuleData = binding.CreateModuleData & {
	settings: {};
	matchResource: string;
};

export class NormalModuleFactory {
	hooks: {
		// TODO: second param resolveData
		resolveForScheme: HookMap<
			AsyncSeriesBailHook<[ResourceDataWithData], true | void>
		>;
		beforeResolve: liteTapable.AsyncSeriesBailHook<[ResolveData], false | void>;
		afterResolve: AsyncSeriesBailHook<[ResolveData], boolean | void>;
		createModule: AsyncSeriesBailHook<[CreateModuleData, {}], void>;
	};
	constructor() {
		this.hooks = {
			// /** @type {AsyncSeriesBailHook<[ResolveData], Module | false | void>} */
			// resolve: new AsyncSeriesBailHook(["resolveData"]),
			// /** @type {HookMap<AsyncSeriesBailHook<[ResourceDataWithData, ResolveData], true | void>>} */
			resolveForScheme: new HookMap(
				() => new AsyncSeriesBailHook(["resourceData"])
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
			afterResolve: new AsyncSeriesBailHook(["resolveData"]),
			// /** @type {AsyncSeriesBailHook<[ResolveData["createData"], ResolveData], Module | void>} */
			createModule: new AsyncSeriesBailHook(["createData", "resolveData"])
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
