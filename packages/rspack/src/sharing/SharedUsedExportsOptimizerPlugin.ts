import type {
  BuiltinPlugin,
  RawSharedUsedExportsOptimizerPluginOptions,
} from '@rspack/binding';
import { BuiltinPluginName } from '@rspack/binding';

import {
  createBuiltinPlugin,
  RspackBuiltinPlugin,
} from '../builtin-plugin/base';
import {
  getFileName,
  type ModuleFederationManifestPluginOptions,
} from '../container/ModuleFederationManifestPlugin';
import {
  normalizeShareScope,
  type NormalizedSharedOptions,
  type ShareScope,
} from './SharePlugin';
import {
  resolveShareKey,
  resolveShareRequest,
  resolveShareScope,
} from './utils';

type OptimizeSharedConfig = {
  request: string;
  issuerLayer?: string;
  shareKey: string;
  shareScope: ShareScope;
  layer?: string;
  treeShaking: boolean;
  usedExports?: string[];
};

export class SharedUsedExportsOptimizerPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.SharedUsedExportsOptimizerPlugin;
  private sharedOptions: NormalizedSharedOptions;
  private injectTreeShakingUsedExports: boolean;
  private manifestOptions: ModuleFederationManifestPluginOptions;
  private shareScope?: ShareScope;

  constructor(
    sharedOptions: NormalizedSharedOptions,
    injectTreeShakingUsedExports?: boolean,
    manifestOptions?: ModuleFederationManifestPluginOptions,
    shareScope?: ShareScope,
  ) {
    super();
    this.sharedOptions = sharedOptions;
    this.injectTreeShakingUsedExports = injectTreeShakingUsedExports ?? true;
    this.manifestOptions = manifestOptions ?? {};
    this.shareScope = shareScope;
  }

  private buildOptions(): RawSharedUsedExportsOptimizerPluginOptions {
    const shared: OptimizeSharedConfig[] = this.sharedOptions.map(
      ([configKey, config]) => ({
        request: resolveShareRequest(config.request, configKey),
        issuerLayer: config.issuerLayer,
        shareKey: resolveShareKey(config.shareKey, configKey),
        shareScope: normalizeShareScope(
          resolveShareScope(config.shareScope, this.shareScope),
          true,
          'SharedUsedExportsOptimizerPlugin',
        ),
        layer: config.layer,
        treeShaking: !!config.treeShaking,
        usedExports: config.treeShaking?.usedExports,
      }),
    );
    const { manifestFileName, statsFileName } = getFileName(
      this.manifestOptions,
    );
    return {
      shared,
      injectTreeShakingUsedExports: this.injectTreeShakingUsedExports,
      manifestFileName,
      statsFileName,
    };
  }

  raw(): BuiltinPlugin | undefined {
    if (!this.sharedOptions.length) {
      return;
    }
    return createBuiltinPlugin(this.name, this.buildOptions());
  }
}
