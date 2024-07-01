import type * as binding from "@rspack/binding";

import * as liteTapable from "@rspack/lite-tapable";
import { ResolveData, ResourceDataWithData } from "./Module";

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
		factorize: liteTapable.AsyncSeriesBailHook<[ResolveData], void>;
		resolve: liteTapable.AsyncSeriesBailHook<[ResolveData], void>;
		afterResolve: liteTapable.AsyncSeriesBailHook<[ResolveData], false | void>;
		createModule: liteTapable.AsyncSeriesBailHook<
			[NormalModuleCreateData, {}],
			void
		>;
	};
	constructor() {
		this.hooks = {
			resolveForScheme: new liteTapable.HookMap(
				() => new liteTapable.AsyncSeriesBailHook(["resourceData"])
			),
			beforeResolve: new liteTapable.AsyncSeriesBailHook(["resolveData"]),
			factorize: new liteTapable.AsyncSeriesBailHook(["resolveData"]),
			resolve: new liteTapable.AsyncSeriesBailHook(["resolveData"]),
			afterResolve: new liteTapable.AsyncSeriesBailHook(["resolveData"]),
			createModule: new liteTapable.AsyncSeriesBailHook([
				"createData",
				"resolveData"
			])
		};
	}
}
