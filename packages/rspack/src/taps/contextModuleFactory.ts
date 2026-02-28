import binding from '@rspack/binding';
import {
  ContextModuleFactoryAfterResolveData,
  ContextModuleFactoryBeforeResolveData,
} from '../Module';
import type { CreatePartialRegisters } from './types';

export const createContextModuleFactoryHooksRegisters: CreatePartialRegisters<
  `ContextModuleFactory`
> = (getCompiler, createTap) => {
  return {
    registerContextModuleFactoryBeforeResolveTaps: createTap(
      binding.RegisterJsTapKind.ContextModuleFactoryBeforeResolve,

      function () {
        return getCompiler().__internal__get_compilation_params()!
          .contextModuleFactory.hooks.beforeResolve;
      },

      function (queried) {
        return async function (
          bindingData: false | binding.JsContextModuleFactoryBeforeResolveData,
        ) {
          const data = bindingData
            ? ContextModuleFactoryBeforeResolveData.__from_binding(bindingData)
            : false;
          const result = await queried.promise(data);
          return result
            ? ContextModuleFactoryBeforeResolveData.__to_binding(result)
            : false;
        };
      },
    ),
    registerContextModuleFactoryAfterResolveTaps: createTap(
      binding.RegisterJsTapKind.ContextModuleFactoryAfterResolve,

      function () {
        return getCompiler().__internal__get_compilation_params()!
          .contextModuleFactory.hooks.afterResolve;
      },

      function (queried) {
        return async function (
          bindingData: false | binding.JsContextModuleFactoryAfterResolveData,
        ) {
          const data = bindingData
            ? ContextModuleFactoryAfterResolveData.__from_binding(bindingData)
            : false;
          const result = await queried.promise(data);
          return result
            ? ContextModuleFactoryAfterResolveData.__to_binding(result)
            : false;
        };
      },
    ),
  };
};
