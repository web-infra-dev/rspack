import { BuiltinPluginName, type JsLoaderContext } from '@rspack/binding';

import type { Compiler } from '../Compiler';
import { runLoaders } from '../loader-runner';
import { create } from './base';

export const JsLoaderRspackPlugin = create(
  BuiltinPluginName.JsLoaderRspackPlugin,
  (compiler: Compiler) => ({
    runner: (context: JsLoaderContext) => runLoaders(compiler, context),
  }),
  /* Not Inheretable */
  'thisCompilation',
);
