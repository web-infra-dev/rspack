import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';

import type { Compiler } from '../Compiler';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export class WebWorkerTemplatePlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.WebWorkerTemplatePlugin;

  raw(compiler: Compiler): BuiltinPlugin | undefined {
    compiler.options.output.chunkLoading = 'import-scripts';
    return createBuiltinPlugin(this.name, undefined);
  }
}
