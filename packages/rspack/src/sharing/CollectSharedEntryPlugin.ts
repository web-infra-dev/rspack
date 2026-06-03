import {
  type BuiltinPlugin,
  BuiltinPluginName,
  type RawCollectShareEntryPluginOptions,
} from '@rspack/binding';
import {
  createBuiltinPlugin,
  RspackBuiltinPlugin,
} from '../builtin-plugin/base';
import type { Compilation } from '../Compilation';
import type { Compiler } from '../Compiler';
import { normalizeConsumeShareOptions } from './ConsumeSharedPlugin';
import {
  createConsumeShareOptions,
  type NormalizedSharedOptions,
  type ShareScope,
} from './SharePlugin';

export type CollectSharedEntryPluginOptions = {
  sharedOptions: NormalizedSharedOptions;
  shareScope?: ShareScope;
};

export type ShareRequestsMap = Record<
  string,
  {
    shareScope: string;
    requests: [string, string][];
  }
>;

const SHARE_ENTRY_ASSET = 'collect-shared-entries.json';
const READ_COLLECTED_SHARED_ENTRY_STAGE = 101;

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

    const readCollectedEntries = (compilation: Compilation) => {
      const asset = compilation.getAsset(SHARE_ENTRY_ASSET);
      if (!asset) return;
      this._collectedEntries = JSON.parse(asset.source.source().toString());
      compilation.deleteAsset(asset.name);
    };

    compiler.hooks.finishMake.tap(
      {
        name: 'CollectSharedEntry',
        stage: READ_COLLECTED_SHARED_ENTRY_STAGE,
      },
      readCollectedEntries,
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
