import {
  type BuiltinPlugin,
  BuiltinPluginName,
  type RawProvideOptions,
} from '@rspack/binding';
import {
  createBuiltinPlugin,
  RspackBuiltinPlugin,
} from '../builtin-plugin/base';
import type { Compiler } from '../Compiler';
import { parseOptions } from '../container/options';
import { ShareRuntimePlugin } from './ShareRuntimePlugin';

export type ProvideSharedPluginOptions<Enhanced extends boolean = false> = {
  provides: Provides<Enhanced>;
  shareScope?: string;
  enhanced?: Enhanced;
};
export type Provides<Enhanced extends boolean> =
  | (ProvidesItem | ProvidesObject<Enhanced>)[]
  | ProvidesObject<Enhanced>;
export type ProvidesItem = string;
export type ProvidesObject<Enhanced extends boolean> = {
  [k: string]: ProvidesConfig<Enhanced> | ProvidesItem;
};
export type ProvidesConfig<Enhanced extends boolean> = Enhanced extends true
  ? ProvidesEnhancedConfig
  : ProvidesV1Config;
type ProvidesV1Config = {
  eager?: boolean;
  shareKey: string;
  shareScope?: string;
  version?: false | string;
};
type ProvidesEnhancedConfig = ProvidesV1Config & ProvidesEnhancedExtraConfig;
type ProvidesEnhancedExtraConfig = {
  singleton?: boolean;
  strictVersion?: boolean;
  requiredVersion?: false | string;
};

export class ProvideSharedPlugin<
  Enhanced extends boolean = false,
> extends RspackBuiltinPlugin {
  name = BuiltinPluginName.ProvideSharedPlugin;
  _provides: [string, Omit<RawProvideOptions, 'key'>][];
  _enhanced?: Enhanced;

  constructor(options: ProvideSharedPluginOptions<Enhanced>) {
    super();
    this._provides = parseOptions(
      options.provides,
      (item) => {
        if (Array.isArray(item))
          throw new Error('Unexpected array of provides');
        return {
          shareKey: item,
          version: undefined,
          shareScope: options.shareScope || 'default',
          eager: false,
        };
      },
      (item) => {
        const raw = {
          shareKey: item.shareKey,
          version: item.version,
          shareScope: item.shareScope || options.shareScope || 'default',
          eager: !!item.eager,
        };
        if (options.enhanced) {
          const enhancedItem: ProvidesConfig<true> = item;
          return {
            ...raw,
            singleton: enhancedItem.singleton,
            requiredVersion: enhancedItem.requiredVersion,
            strictVersion: enhancedItem.strictVersion,
          };
        }
        return raw;
      },
    );
    this._enhanced = options.enhanced;
  }

  raw(compiler: Compiler): BuiltinPlugin {
    new ShareRuntimePlugin(this._enhanced ?? false).apply(compiler);

    const rawOptions: RawProvideOptions[] = this._provides.map(([key, v]) => ({
      key,
      ...v,
    }));
    return createBuiltinPlugin(this.name, rawOptions);
  }
}
