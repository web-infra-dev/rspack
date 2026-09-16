import binding from '@rspack/binding';
import { commitCustomFieldsToRust } from '../BuildInfo';
import { createLoaderContext } from '../loader-runner';
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
    (queried) => (context: binding.JsLoaderContext) => {
      const compiler = getCompiler();
      const dependencies = new LoaderDependenciesState(
        context.state.dependencies,
      );
      const loaderContext = createLoaderContext(
        compiler,
        context,
        dependencies,
      );
      queried.call(loaderContext, loaderContext._module);
      dependencies.mergeChanges();
      if (compiler.options.cache) {
        commitCustomFieldsToRust(context.meta._module.buildInfo);
      }
      return context.state;
    },
  ),
});
