import {
  type BuiltinPlugin,
  BuiltinPluginName,
  type RawCollectShareEntryPluginOptions,
} from '@rspack/binding';
import {
  createBuiltinPlugin,
  RspackBuiltinPlugin,
} from '../builtin-plugin/base';
import type { Compiler } from '../Compiler';
import { normalizeConsumeShareOptions } from './ConsumeSharedPlugin';
import {
  createConsumeShareOptions,
  type NormalizedSharedOptions,
} from './SharePlugin';

export type CollectSharedEntryPluginOptions = {
  sharedOptions: NormalizedSharedOptions;
  shareScope?: string;
};

export type ShareRequestsMap = Record<
  string,
  {
    shareScope: string;
    requests: [string, string][];
  }
>;

const SHARE_ENTRY_ASSET = 'collect-shared-entries.json';
export class CollectSharedEntryPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.CollectSharedEntryPlugin;
  sharedOptions: NormalizedSharedOptions;
  private _collectedEntries: ShareRequestsMap;

  constructor(options: CollectSharedEntryPluginOptions) {
    super();
    const { sharedOptions } = options;

    this.sharedOptions = sharedOptions;
    this._collectedEntries = {};
  }

  getData() {
    return this._collectedEntries;
  }

  getFilename() {
    return SHARE_ENTRY_ASSET;
  }

  apply(compiler: Compiler) {
    super.apply(compiler);

    compiler.hooks.thisCompilation.tap(
      'Collect shared entry',
      (compilation) => {
        compilation.hooks.processAssets.tap(
          {
            name: 'CollectSharedEntry',
            stage:
              compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE,
          },
          () => {
            compilation.getAssets().forEach((asset) => {
              if (asset.name === SHARE_ENTRY_ASSET) {
                this._collectedEntries = JSON.parse(
                  asset.source.source().toString(),
                );
              }
              compilation.deleteAsset(asset.name);
            });
          },
        );
      },
    );
  }

  raw(): BuiltinPlugin {
    const consumeShareOptions = createConsumeShareOptions(this.sharedOptions);
    const normalizedConsumeShareOptions =
      normalizeConsumeShareOptions(consumeShareOptions);
    const rawOptions: RawCollectShareEntryPluginOptions = {
      consumes: normalizedConsumeShareOptions.map(([key, v]) => ({
        key,
        ...v,
      })),
      filename: this.getFilename(),
    };
    return createBuiltinPlugin(this.name, rawOptions);
  }
}
