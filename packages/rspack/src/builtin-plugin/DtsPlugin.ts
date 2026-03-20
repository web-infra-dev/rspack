import { BuiltinPluginName } from '@rspack/binding';
import type { Compiler } from '../Compiler';

export interface DtsPluginOptions {
  entries: Record<string, string>;
  filename?: string;
  externals?: string[];
}

const DEFAULT_FILENAME = '[name].d.ts';

export class DtsPlugin {
  options: DtsPluginOptions;

  constructor(options: DtsPluginOptions) {
    this.options = options;
  }

  apply(compiler: Compiler) {
    compiler.options.module.rules.unshift({
      test: /\.d\.(?:c|m)?ts$/,
      type: 'dts',
    });

    const rawOptions = {
      entries: Object.entries(this.options.entries).map(([name, request]) => ({
        name,
        request,
      })),
      filename: this.options.filename ?? DEFAULT_FILENAME,
      externals: this.options.externals ?? [],
    };

    compiler.__internal__registerBuiltinPlugin({
      name: (BuiltinPluginName as any).DtsRspackPlugin,
      options: rawOptions,
    });
  }
}
