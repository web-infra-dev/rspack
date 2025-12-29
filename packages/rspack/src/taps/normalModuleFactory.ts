import binding from '@rspack/binding';
import type { NormalModuleCreateData } from '../NormalModuleFactory';
import type { CreatePartialRegisters } from './types';

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
        return async function (resolveData: binding.JsResolveData) {
          const ret = await queried.promise(resolveData);
          return [ret, resolveData];
        };
      },
    ),
    registerNormalModuleFactoryFactorizeTaps: createTap(
      binding.RegisterJsTapKind.NormalModuleFactoryFactorize,

      function () {
        return getCompiler().__internal__get_compilation_params()!
          .normalModuleFactory.hooks.factorize;
      },

      function (queried) {
        return async function (resolveData: binding.JsResolveData) {
          await queried.promise(resolveData);
          return resolveData;
        };
      },
    ),
    registerNormalModuleFactoryResolveTaps: createTap(
      binding.RegisterJsTapKind.NormalModuleFactoryResolve,

      function () {
        return getCompiler().__internal__get_compilation_params()!
          .normalModuleFactory.hooks.resolve;
      },

      function (queried) {
        return async function (resolveData: binding.JsResolveData) {
          await queried.promise(resolveData);
          return resolveData;
        };
      },
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
      },
    ),
    registerNormalModuleFactoryAfterResolveTaps: createTap(
      binding.RegisterJsTapKind.NormalModuleFactoryAfterResolve,

      function () {
        return getCompiler().__internal__get_compilation_params()!
          .normalModuleFactory.hooks.afterResolve;
      },

      function (queried) {
        return async function (resolveData: binding.JsResolveData) {
          const ret = await queried.promise(resolveData);
          return [ret, resolveData];
        };
      },
    ),
    registerNormalModuleFactoryCreateModuleTaps: createTap(
      binding.RegisterJsTapKind.NormalModuleFactoryCreateModule,

      function () {
        return getCompiler().__internal__get_compilation_params()!
          .normalModuleFactory.hooks.createModule;
      },

      function (queried) {
        return async function (
          args: binding.JsNormalModuleFactoryCreateModuleArgs,
        ) {
          const data: NormalModuleCreateData = {
            ...args,
            settings: {},
          };
          await queried.promise(data, {});
        };
      },
    ),
  };
};
