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
import type { NormalizedSharedOptions } from './SharePlugin';

type OptimizeSharedConfig = {
  shareKey: string;
  treeShaking: boolean;
  usedExports?: string[];
};

export class SharedUsedExportsOptimizerPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.SharedUsedExportsOptimizerPlugin;
  private sharedOptions: NormalizedSharedOptions;
  private injectTreeShakingUsedExports: boolean;
  private manifestOptions: ModuleFederationManifestPluginOptions;

  constructor(
    sharedOptions: NormalizedSharedOptions,
    injectTreeShakingUsedExports?: boolean,
    manifestOptions?: ModuleFederationManifestPluginOptions,
  ) {
    super();
    this.sharedOptions = sharedOptions;
    this.injectTreeShakingUsedExports = injectTreeShakingUsedExports ?? true;
    this.manifestOptions = manifestOptions ?? {};
  }

  private buildOptions(): RawSharedUsedExportsOptimizerPluginOptions {
    const shared: OptimizeSharedConfig[] = this.sharedOptions.map(
      ([shareKey, config]) => ({
        shareKey,
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
