import {
  BuiltinPluginName,
  moduleRevokeDirectAccess,
  type JsLoaderContext,
} from '@rspack/binding';

import type { Compiler } from '../Compiler';
import { runLoaders } from '../loader-runner';
import { getCompilerHandle } from '../loader-runner/service';
import { create } from './base';

export const JsLoaderRspackPlugin = create(
  BuiltinPluginName.JsLoaderRspackPlugin,
  (compiler: Compiler) => ({
    mainObjectHandle: getCompilerHandle(compiler),
    runner: async (context: JsLoaderContext) => {
      const module = context._module!;
      try {
        return await runLoaders(compiler, context);
      } finally {
        moduleRevokeDirectAccess(module);
      }
    },
  }),
  /* Not Inheretable */
  'thisCompilation',
);
