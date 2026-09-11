import binding from '@rspack/binding';
import { commitCustomFieldsToRust } from '../BuildInfo';
import { createLoaderContext } from '../loader-runner';
import {
  LoaderContextState,
  toLoaderContextError,
} from '../loader-runner/context';
import { LoaderDependenciesState } from '../loader-runner/dependencies';
import { NormalModule } from '../NormalModule';
import type { CreatePartialRegisters } from './types';

export const createNormalModuleHooksRegisters: CreatePartialRegisters<
  'NormalModuleLoader'
> = (getCompiler, createTap) => ({
  registerNormalModuleLoaderTaps: createTap(
    binding.RegisterJsTapKind.NormalModuleLoader,
    () =>
      NormalModule.getCompilationHooks(
        getCompiler().__internal__get_compilation()!,
      ).loader,
    (queried) => (nativeContext: binding.JsLoaderHookContext) => {
      try {
        const context = new LoaderContextState(nativeContext);
        const compiler = getCompiler();
        const dependencies = new LoaderDependenciesState(context.dependencies);
        const loaderContext = createLoaderContext(
          compiler,
          context,
          dependencies,
        );
        queried.call(loaderContext, loaderContext._module);
        dependencies.mergeChanges();
        if (compiler.options.cache) {
          commitCustomFieldsToRust(context._module.buildInfo);
        }
      } catch (error) {
        nativeContext.state.error = toLoaderContextError(error);
      }
      return nativeContext.state;
    },
  ),
});
