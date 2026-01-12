import type { Compiler } from '../Compiler';
import type { ModuleFederationPluginOptions } from '../container/ModuleFederationPlugin';
import { IndependentSharedPlugin } from './IndependentSharedPlugin';
import { SharedUsedExportsOptimizerPlugin } from './SharedUsedExportsOptimizerPlugin';
import { normalizeSharedOptions } from './SharePlugin';

export interface TreeshakingSharedPluginOptions {
  mfConfig: ModuleFederationPluginOptions;
  reShake?: boolean;
}

export class TreeShakingSharedPlugin {
  mfConfig: ModuleFederationPluginOptions;
  outputDir: string;
  reShake?: boolean;
  private _independentSharePlugin?: IndependentSharedPlugin;

  name = 'TreeShakingSharedPlugin';
  constructor(options: TreeshakingSharedPluginOptions) {
    const { mfConfig, reShake } = options;
    this.mfConfig = mfConfig;
    this.outputDir = mfConfig.treeShakingSharedDir || 'independent-packages';
    this.reShake = Boolean(reShake);
  }

  apply(compiler: Compiler) {
    const { mfConfig, outputDir, reShake } = this;
    const { name, shared, library, treeShakingSharedPlugins } = mfConfig;
    if (!shared) {
      return;
    }
    const sharedOptions = normalizeSharedOptions(shared);
    if (!sharedOptions.length) {
      return;
    }

    if (
      sharedOptions.some(
        ([_, config]) => config.treeShaking && config.import !== false,
      )
    ) {
      if (!reShake) {
        new SharedUsedExportsOptimizerPlugin(
          sharedOptions,
          mfConfig.injectTreeShakingUsedExports,
          mfConfig.manifest,
        ).apply(compiler);
      }
      this._independentSharePlugin = new IndependentSharedPlugin({
        name: name,
        shared: shared,
        outputDir,
        plugins:
          treeShakingSharedPlugins?.map((p) => {
            const _constructor = require(p);
            return new _constructor();
          }) || [],
        treeShaking: reShake,
        library,
        manifest: mfConfig.manifest,
        treeShakingSharedExcludePlugins:
          mfConfig.treeShakingSharedExcludePlugins,
      });
      this._independentSharePlugin.apply(compiler);
    }
  }

  get buildAssets() {
    return this._independentSharePlugin?.buildAssets || {};
  }
}
