import {
  type BuiltinPlugin,
  BuiltinPluginName,
  type RawConsumeSharedPluginOptions,
} from '@rspack/binding';
import {
  createBuiltinPlugin,
  RspackBuiltinPlugin,
} from '../builtin-plugin/base';
import type { Compiler } from '../Compiler';
import { parseOptions } from '../container/options';
import { normalizeShareScope, type ShareScope } from './SharePlugin';
import { ShareRuntimePlugin } from './ShareRuntimePlugin';
import {
  isRequiredVersion,
  resolveShareKey,
  resolveShareRequest,
  resolveShareScope,
} from './utils';

type ConsumeSharedPluginBaseOptions<Enhanced extends boolean> = {
  consumes: Consumes<Enhanced>;
  shareScope?: ShareScope;
};
export type ConsumeSharedPluginOptions<Enhanced extends boolean = boolean> = [
  Enhanced,
] extends [true]
  ? ConsumeSharedPluginBaseOptions<true> & { enhanced: true }
  : [Enhanced] extends [false]
    ? ConsumeSharedPluginBaseOptions<false> & { enhanced?: false }
    : | (ConsumeSharedPluginBaseOptions<false> & { enhanced?: false })
      | (ConsumeSharedPluginBaseOptions<true> & { enhanced: true });
export type Consumes<Enhanced extends boolean = boolean> =
  (ConsumesItem | ConsumesObject<Enhanced>)[] | ConsumesObject<Enhanced>;
export type ConsumesItem = string;
export type ConsumesObject<Enhanced extends boolean = boolean> = {
  [k: string]: ConsumesConfig<Enhanced> | ConsumesItem;
};
type ConsumesV1Config = {
  eager?: boolean;
  import?: false | ConsumesItem;
  packageName?: string;
  requiredVersion?: false | string;
  shareKey?: string;
  shareScope?: ShareScope;
  singleton?: boolean;
  strictVersion?: boolean;
  treeShakingMode?: 'server-calc' | 'runtime-infer';
};
type ConsumesEnhancedConfig = ConsumesV1Config & {
  issuerLayer?: string;
  layer?: string;
  request?: string;
};
export type ConsumesConfig<Enhanced extends boolean = boolean> = [
  Enhanced,
] extends [true]
  ? ConsumesEnhancedConfig
  : ConsumesV1Config;

export function normalizeConsumeShareOptions<
  Enhanced extends boolean = boolean,
>(consumes: Consumes<Enhanced>, shareScope?: ShareScope, enhanced?: Enhanced) {
  return parseOptions(
    consumes,
    (item, key) => {
      if (Array.isArray(item)) throw new Error('Unexpected array in options');
      const result =
        item === key || !isRequiredVersion(item)
          ? // item is a request/key
            {
              import: key,
              shareScope: normalizeShareScope(
                resolveShareScope(undefined, shareScope),
                !!enhanced,
                'ConsumeSharedPlugin',
              ),
              shareKey: key,
              requiredVersion: undefined,
              packageName: undefined,
              strictVersion: false,
              singleton: false,
              eager: false,
              issuerLayer: undefined,
              layer: undefined,
              request: key,
              treeShakingMode: undefined,
            }
          : // key is a request/key
            // item is a version
            {
              import: key,
              shareScope: normalizeShareScope(
                resolveShareScope(undefined, shareScope),
                !!enhanced,
                'ConsumeSharedPlugin',
              ),
              shareKey: key,
              requiredVersion: item,
              strictVersion: true,
              packageName: undefined,
              singleton: false,
              eager: false,
              issuerLayer: undefined,
              layer: undefined,
              request: key,
              treeShakingMode: undefined,
            };
      return result;
    },
    (item, key) => {
      const enhancedItem = item as ConsumesEnhancedConfig;
      if (!enhanced) {
        const unsupported = ['request', 'issuerLayer', 'layer'].find(
          (field) =>
            enhancedItem[field as keyof ConsumesEnhancedConfig] !== undefined,
        );
        if (unsupported) {
          throw new Error(
            `[ConsumeSharedPlugin] ${unsupported} requires enhanced=true`,
          );
        }
      }
      const request = enhanced
        ? resolveShareRequest(enhancedItem.request, key)
        : key;
      return {
        import: item.import === false ? undefined : item.import || request,
        shareScope: normalizeShareScope(
          resolveShareScope(item.shareScope, shareScope),
          !!enhanced,
          'ConsumeSharedPlugin',
        ),
        shareKey: resolveShareKey(item.shareKey, key),
        requiredVersion: item.requiredVersion,
        strictVersion:
          typeof item.strictVersion === 'boolean'
            ? item.strictVersion
            : item.import !== false && !item.singleton,
        packageName: item.packageName,
        singleton: !!item.singleton,
        eager: !!item.eager,
        issuerLayer: enhanced ? enhancedItem.issuerLayer : undefined,
        layer: enhanced ? enhancedItem.layer : undefined,
        request,
        treeShakingMode: item.treeShakingMode,
      };
    },
  );
}

export class ConsumeSharedPlugin<
  Enhanced extends boolean = boolean,
> extends RspackBuiltinPlugin {
  name = BuiltinPluginName.ConsumeSharedPlugin;
  _options;

  constructor(options: ConsumeSharedPluginOptions<Enhanced>) {
    super();
    this._options = {
      consumes: normalizeConsumeShareOptions(
        options.consumes,
        options.shareScope,
        options.enhanced,
      ),
      enhanced: options.enhanced ?? false,
    };
  }

  raw(compiler: Compiler): BuiltinPlugin {
    new ShareRuntimePlugin(this._options.enhanced).apply(compiler);

    const rawOptions: RawConsumeSharedPluginOptions = {
      consumes: this._options.consumes.map(([key, v]) => ({
        key,
        ...v,
      })),
      enhanced: this._options.enhanced,
    };
    return createBuiltinPlugin(this.name, rawOptions);
  }
}
