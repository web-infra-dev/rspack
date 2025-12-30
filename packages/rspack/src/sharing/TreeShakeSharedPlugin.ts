import type { Compiler } from '../Compiler';
import type { ModuleFederationPluginOptions } from '../container/ModuleFederationPlugin';
import { IndependentSharedPlugin } from './IndependentSharedPlugin';
import { SharedUsedExportsOptimizerPlugin } from './SharedUsedExportsOptimizerPlugin';
import { normalizeSharedOptions } from './SharePlugin';

export interface TreeshakeSharedPluginOptions {
  mfConfig: ModuleFederationPluginOptions;
  reShake?: boolean;
}

export class TreeShakeSharedPlugin {
  mfConfig: ModuleFederationPluginOptions;
  outputDir: string;
  reShake?: boolean;
  private _independentSharePlugin?: IndependentSharedPlugin;

  name = 'TreeShakeSharedPlugin';
  constructor(options: TreeshakeSharedPluginOptions) {
    const { mfConfig, reShake } = options;
    this.mfConfig = mfConfig;
    this.outputDir = mfConfig.independentShareDir || 'independent-packages';
    this.reShake = Boolean(reShake);
  }

  apply(compiler: Compiler) {
    const { mfConfig, outputDir, reShake } = this;
    const { name, shared, library, treeshakeSharedPlugins } = mfConfig;
    if (!shared) {
      return;
    }
    const sharedOptions = normalizeSharedOptions(shared);
    if (!sharedOptions.length) {
      return;
    }

    if (
      sharedOptions.some(
        ([_, config]) => config.treeshake && config.import !== false,
      )
    ) {
      if (!reShake) {
        new SharedUsedExportsOptimizerPlugin(
          sharedOptions,
          mfConfig.injectUsedExports,
          mfConfig.manifest,
        ).apply(compiler);
      }
      this._independentSharePlugin = new IndependentSharedPlugin({
        name: name,
        shared: shared,
        outputDir,
        plugins: treeshakeSharedPlugins?.map((p) => require(p)) || [],
        treeshake: reShake,
        library,
        manifest: mfConfig.manifest,
        treeshakeSharedExcludePlugins: mfConfig.treeshakeSharedExcludePlugins,
      });
      this._independentSharePlugin.apply(compiler);
    }
  }

  get buildAssets() {
    return this._independentSharePlugin?.buildAssets || {};
  }
}
