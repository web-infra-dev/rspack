import * as binding from "@rspack/binding";
import type { ResolveData } from "../Module";
import type { NormalModuleCreateData } from "../NormalModuleFactory";
import type { CreatePartialRegisters } from "./types";

export const createNormalModuleFactoryHooksRegisters: CreatePartialRegisters<
	`NormalModuleFactory`
> = (getCompiler, createTap, createMapTap) => {
	return {
		registerNormalModuleFactoryBeforeResolveTaps: createTap(
			binding.RegisterJsTapKind.NormalModuleFactoryBeforeResolve,

			function () {
				return getCompiler().__internal__get_compilation_params()!
					.normalModuleFactory.hooks.beforeResolve;
			},

			function (queried) {
				return async function (resolveData: binding.JsBeforeResolveArgs) {
					const normalizedResolveData: ResolveData = {
						contextInfo: {
							issuer: resolveData.issuer,
							issuerLayer: resolveData.issuerLayer ?? null
						},
						request: resolveData.request,
						context: resolveData.context,
						fileDependencies: [],
						missingDependencies: [],
						contextDependencies: []
					};
					const ret = await queried.promise(normalizedResolveData);
					resolveData.request = normalizedResolveData.request;
					resolveData.context = normalizedResolveData.context;
					return [ret, resolveData];
				};
			}
		),
		registerNormalModuleFactoryFactorizeTaps: createTap(
			binding.RegisterJsTapKind.NormalModuleFactoryFactorize,

			function () {
				return getCompiler().__internal__get_compilation_params()!
					.normalModuleFactory.hooks.factorize;
			},

			function (queried) {
				return async function (resolveData: binding.JsFactorizeArgs) {
					const normalizedResolveData: ResolveData = {
						contextInfo: {
							issuer: resolveData.issuer,
							issuerLayer: resolveData.issuerLayer ?? null
						},
						request: resolveData.request,
						context: resolveData.context,
						fileDependencies: [],
						missingDependencies: [],
						contextDependencies: []
					};
					await queried.promise(normalizedResolveData);
					resolveData.request = normalizedResolveData.request;
					resolveData.context = normalizedResolveData.context;
					return resolveData;
				};
			}
		),
		registerNormalModuleFactoryResolveTaps: createTap(
			binding.RegisterJsTapKind.NormalModuleFactoryResolve,

			function () {
				return getCompiler().__internal__get_compilation_params()!
					.normalModuleFactory.hooks.resolve;
			},

			function (queried) {
				return async function (resolveData: binding.JsFactorizeArgs) {
					const normalizedResolveData: ResolveData = {
						contextInfo: {
							issuer: resolveData.issuer,
							issuerLayer: resolveData.issuerLayer ?? null
						},
						request: resolveData.request,
						context: resolveData.context,
						fileDependencies: [],
						missingDependencies: [],
						contextDependencies: []
					};
					await queried.promise(normalizedResolveData);
					resolveData.request = normalizedResolveData.request;
					resolveData.context = normalizedResolveData.context;
					return resolveData;
				};
			}
		),
		registerNormalModuleFactoryResolveForSchemeTaps: createMapTap(
			binding.RegisterJsTapKind.NormalModuleFactoryResolveForScheme,

			function () {
				return getCompiler().__internal__get_compilation_params()!
					.normalModuleFactory.hooks.resolveForScheme;
			},

			function (queried) {
				return async function (args: binding.JsResolveForSchemeArgs) {
					const ret = await queried.for(args.scheme).promise(args.resourceData);
					return [ret, args.resourceData];
				};
			}
		),
		registerNormalModuleFactoryAfterResolveTaps: createTap(
			binding.RegisterJsTapKind.NormalModuleFactoryAfterResolve,

			function () {
				return getCompiler().__internal__get_compilation_params()!
					.normalModuleFactory.hooks.afterResolve;
			},

			function (queried) {
				return async function (arg: binding.JsAfterResolveData) {
					const data: ResolveData = {
						contextInfo: {
							issuer: arg.issuer,
							issuerLayer: arg.issuerLayer ?? null
						},
						request: arg.request,
						context: arg.context,
						fileDependencies: arg.fileDependencies,
						missingDependencies: arg.missingDependencies,
						contextDependencies: arg.contextDependencies,
						createData: arg.createData
					};
					const ret = await queried.promise(data);
					return [ret, data.createData];
				};
			}
		),
		registerNormalModuleFactoryCreateModuleTaps: createTap(
			binding.RegisterJsTapKind.NormalModuleFactoryCreateModule,

			function () {
				return getCompiler().__internal__get_compilation_params()!
					.normalModuleFactory.hooks.createModule;
			},

			function (queried) {
				return async function (
					args: binding.JsNormalModuleFactoryCreateModuleArgs
				) {
					const data: NormalModuleCreateData = {
						...args,
						settings: {}
					};
					await queried.promise(data, {});
				};
			}
		)
	};
};
