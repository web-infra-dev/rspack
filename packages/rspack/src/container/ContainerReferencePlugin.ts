import {
  type BuiltinPlugin,
  BuiltinPluginName,
  type RawContainerReferencePluginOptions,
} from '@rspack/binding';
import {
  createBuiltinPlugin,
  RspackBuiltinPlugin,
} from '../builtin-plugin/base';
import { ExternalsPlugin } from '../builtin-plugin/ExternalsPlugin';
import type { Compiler } from '../Compiler';
import type { ExternalsType } from '../config';
import { ShareRuntimePlugin } from '../sharing/ShareRuntimePlugin';
import { parseOptions } from './options';

export type ContainerReferencePluginOptions = {
  remoteType: ExternalsType;
  remotes: Remotes;
  shareScope?: string | string[];
  enhanced?: boolean;
};
export type Remotes = (RemotesItem | RemotesObject)[] | RemotesObject;
export type RemotesItem = string;
export type RemotesItems = RemotesItem[];
export type RemotesObject = {
  [k: string]: RemotesConfig | RemotesItem | RemotesItems;
};
export type RemotesConfig = {
  external: RemotesItem | RemotesItems;
  shareScope?: string | string[];
};

export class ContainerReferencePlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.ContainerReferencePlugin;
  _options;

  constructor(options: ContainerReferencePluginOptions) {
    super();
    this._options = {
      remoteType: options.remoteType,
      remotes: parseOptions(
        options.remotes,
        (item) => ({
          external: Array.isArray(item) ? item : [item],
          shareScope: options.shareScope || 'default',
        }),
        (item) => ({
          external: Array.isArray(item.external)
            ? item.external
            : [item.external],
          shareScope: item.shareScope || options.shareScope || 'default',
        }),
      ),
      enhanced: options.enhanced ?? false,
    };
  }

  raw(compiler: Compiler): BuiltinPlugin {
    const { remoteType, remotes } = this._options;
    const remoteExternals: Record<string, string> = {};
    const importExternals: Record<string, string> = {};
    for (const [key, config] of remotes) {
      let i = 0;
      for (const external of config.external) {
        if (external.startsWith('internal ')) continue;
        const request = `webpack/container/reference/${key}${i ? `/fallback-${i}` : ''}`;
        // In ESM output, `externalsType: "module"` emits a static `import * as ... from "..."`
        // which can create a circular dependency for relative remotes (notably self-remotes like
        // `containerB: "./container.mjs"` with `runtimeChunk: "single"`). Prefer dynamic `import()`
        // for those to break the cycle.
        if (
          (remoteType === 'module' || remoteType === 'module-import') &&
          external.startsWith('.')
        ) {
          importExternals[request] = external;
        } else {
          remoteExternals[request] = external;
        }
        i++;
      }
    }
    new ExternalsPlugin(remoteType, remoteExternals, true).apply(compiler);
    if (Object.keys(importExternals).length > 0) {
      new ExternalsPlugin('import', importExternals, true).apply(compiler);
    }
    new ShareRuntimePlugin(this._options.enhanced).apply(compiler);

    const rawOptions: RawContainerReferencePluginOptions = {
      remoteType: this._options.remoteType,
      remotes: this._options.remotes.map(([key, r]) => ({ key, ...r })),
      enhanced: this._options.enhanced,
    };
    return createBuiltinPlugin(this.name, rawOptions);
  }
}
