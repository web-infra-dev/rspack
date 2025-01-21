import * as binding from "@rspack/binding";
import type { Compiler } from "../Compiler";
import {
	ContextModuleFactoryAfterResolveData,
	ContextModuleFactoryBeforeResolveData
} from "../Module";

type ContextModuleFactoryRegisterJsTapKeys =
	`registerContextModuleFactory${string}Taps`;
type ContextModuleFactoryRegisterTapKeys<T> =
	T extends keyof binding.RegisterJsTaps
		? T extends ContextModuleFactoryRegisterJsTapKeys
			? T
			: never
		: never;
type ContextModuleFactoryTaps = {
	[K in ContextModuleFactoryRegisterTapKeys<
		keyof binding.RegisterJsTaps
	>]: binding.RegisterJsTaps[K];
};

export function createContextModuleFactoryHooksRegisters(
	getCompiler: () => Compiler,
	createTap: Compiler["__internal__create_hook_register_taps"],
	_createMapTap: Compiler["__internal__create_hook_map_register_taps"]
): ContextModuleFactoryTaps {
	return {
		registerContextModuleFactoryBeforeResolveTaps: createTap(
			binding.RegisterJsTapKind.ContextModuleFactoryBeforeResolve,

			function () {
				return getCompiler().__internal__get_compilation_params()!
					.contextModuleFactory.hooks.beforeResolve;
			},

			function (queried) {
				return async function (
					bindingData: false | binding.JsContextModuleFactoryBeforeResolveData
				) {
					const data = bindingData
						? ContextModuleFactoryBeforeResolveData.__from_binding(bindingData)
						: false;
					const result = await queried.promise(data);
					return result
						? ContextModuleFactoryBeforeResolveData.__to_binding(result)
						: false;
				};
			}
		),
		registerContextModuleFactoryAfterResolveTaps: createTap(
			binding.RegisterJsTapKind.ContextModuleFactoryAfterResolve,

			function () {
				return getCompiler().__internal__get_compilation_params()!
					.contextModuleFactory.hooks.afterResolve;
			},

			function (queried) {
				return async function (
					bindingData: false | binding.JsContextModuleFactoryAfterResolveData
				) {
					const data = bindingData
						? ContextModuleFactoryAfterResolveData.__from_binding(bindingData)
						: false;
					const result = await queried.promise(data);
					return result
						? ContextModuleFactoryAfterResolveData.__to_binding(result)
						: false;
				};
			}
		)
	};
}
