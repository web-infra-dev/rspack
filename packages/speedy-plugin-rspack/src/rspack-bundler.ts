import { Callback, IBundlerBase, ISpeedyBundler, WebpackStats } from '@speedy-js/speedy-types';
import { Rspack, RspackPlugin } from '@rspack/core';

function adapterPlugin(compiler: ISpeedyBundler): RspackPlugin {
  return {
    name: 'adapterPlugin',
    load: async (...args) => {
      console.log('load args:', args);
    },
    resolve: async (...args) => {
      console.log('resolve args:', args);
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
    this.instance.build();
  }
  async reBuild(paths: string[]): Promise<void> {
    this.instance.rebuild(paths);
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
