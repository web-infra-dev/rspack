import { createRequire } from 'node:module';
import type { Compiler } from '../Compiler';
import type { ModuleFederationPluginOptions } from '../container/ModuleFederationPlugin';
import {
  IndependentSharedPlugin,
  type ShareFallback,
} from './IndependentSharedPlugin';
import { SharedUsedExportsOptimizerPlugin } from './SharedUsedExportsOptimizerPlugin';
import { normalizeSharedOptions } from './SharePlugin';

const require = createRequire(import.meta.url);

export interface TreeshakingSharedPluginOptions {
  mfConfig: ModuleFederationPluginOptions;
  secondary?: boolean;
  onBuildAssets?: (buildAssets: ShareFallback) => void;
}

export class TreeShakingSharedPlugin {
  mfConfig: ModuleFederationPluginOptions;
  outputDir: string;
  secondary?: boolean;
  onBuildAssets?: (buildAssets: ShareFallback) => void;
  private _independentSharePlugin?: IndependentSharedPlugin;

  name = 'TreeShakingSharedPlugin';
  constructor(options: TreeshakingSharedPluginOptions) {
    const { mfConfig, secondary, onBuildAssets } = options;
    this.mfConfig = mfConfig;
    this.outputDir = mfConfig.treeShakingSharedDir || 'independent-packages';
    this.secondary = Boolean(secondary);
    this.onBuildAssets = onBuildAssets;
  }

  apply(compiler: Compiler) {
    const { mfConfig, outputDir, secondary } = this;
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
      if (!secondary) {
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
        treeShaking: secondary,
        library,
        manifest: mfConfig.manifest,
        treeShakingSharedExcludePlugins:
          mfConfig.treeShakingSharedExcludePlugins,
        onBuildAssets: this.onBuildAssets,
      });
      this._independentSharePlugin.apply(compiler);
    }
  }

  get buildAssets() {
    return this._independentSharePlugin?.buildAssets || {};
  }
}
