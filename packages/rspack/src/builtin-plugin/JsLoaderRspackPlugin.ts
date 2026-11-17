import { BuiltinPluginName } from '@rspack/binding';

import type { Compiler } from '../Compiler';
import { runLoaders } from '../loader-runner';
import { getCompilerHandle } from '../loader-runner/service';
import { create } from './base';

export const JsLoaderRspackPlugin = create(
  BuiltinPluginName.JsLoaderRspackPlugin,
  (compiler: Compiler) => ({
    mainObjectHandle: getCompilerHandle(compiler),
    runner: runLoaders.bind(null, compiler),
  }),
  /* Not Inheretable */
  'thisCompilation',
);
