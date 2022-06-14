import { Callback, IBundlerBase, ISpeedyBundler, WebpackStats } from '@speedy-js/speedy-types';
import { Rspack, RspackPlugin } from '@rspack/core';
import path from 'path';
function adapterPlugin(compiler: ISpeedyBundler): RspackPlugin {
  return {
    name: 'adapterPlugin',
    load: async (args) => {
      let result = await compiler.hooks.load.promise({
        path: args,
      });
      console.log('load args:', args, result);
      if (result) {
        return {
          loader: result.loader as any,
        };
      } else {
        return undefined;
      }
    },
    resolve: async (id, importer) => {
      let result = await compiler.hooks.resolve.promise({
        importer: importer!,
        path: id,
        resolveDir: importer ? path.dirname(importer!) : compiler.config.root,
        kind: 'require-call',
      });
    },
    buildStart: async (...args) => {
      console.log('start:', args);
      await compiler.hooks.startCompilation.promise();
    },
    buildEnd: async (assets) => {
      console.log(assets);
      await compiler.hooks.endCompilation.promise();
    },
  };
}
export class RspackBundler implements IBundlerBase {
  compiler: ISpeedyBundler;
  instance!: Rspack;
  constructor(compiler: ISpeedyBundler) {
    this.compiler = compiler;
  }
  async build(): Promise<void> {
    const rspackConfig = {};
    const { input, mode, output } = this.compiler.config;
    this.instance = new Rspack({
      entries: this.compiler.config.input as any,
      mode,
      output: { outdir: output.path },
      plugins: [adapterPlugin(this.compiler)],
    });
    console.log('start build');
    await this.instance.build();
    console.log('finish build');
  }
  async reBuild(paths: string[]): Promise<void> {
    await this.instance.rebuild(paths);
  }
  close(callack?: Callback): void {
    console.log('not implmented');
  }
  getStats(): WebpackStats {
    return {} as any;
  }
  setStats(stats: WebpackStats): void {
    console.log('setStats');
  }
  shouldRebuild(paths: string[]): boolean {
    return true;
  }
  canRebuild(): boolean {
    return true;
  }
}
