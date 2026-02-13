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
import { ShareRuntimePlugin } from './ShareRuntimePlugin';
import { isRequiredVersion } from './utils';

export type ConsumeSharedPluginOptions = {
  consumes: Consumes;
  shareScope?: string | string[];
  enhanced?: boolean;
};
export type Consumes = (ConsumesItem | ConsumesObject)[] | ConsumesObject;
export type ConsumesItem = string;
export type ConsumesObject = {
  [k: string]: ConsumesConfig | ConsumesItem;
};
export type ConsumesConfig = {
  eager?: boolean;
  import?: false | ConsumesItem;
  issuerLayer?: string;
  layer?: string;
  packageName?: string;
  request?: string;
  requiredVersion?: false | string;
  shareKey?: string;
  shareScope?: string | string[];
  singleton?: boolean;
  strictVersion?: boolean;
  treeShakingMode?: 'server-calc' | 'runtime-infer';
};

export function normalizeConsumeShareOptions(
  consumes: Consumes,
  shareScope?: string | string[],
  enhanced?: boolean,
) {
  return parseOptions(
    consumes,
    (item, key) => {
      if (Array.isArray(item)) throw new Error('Unexpected array in options');
      const result =
        item === key || !isRequiredVersion(item)
          ? // item is a request/key
            {
              import: key,
              shareScope: shareScope || 'default',
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
              shareScope: shareScope || 'default',
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
      const request = item.request || key;
      return {
        import: item.import === false ? undefined : item.import || request,
        shareScope: item.shareScope || shareScope || 'default',
        shareKey: item.shareKey || request,
        requiredVersion: item.requiredVersion,
        strictVersion:
          typeof item.strictVersion === 'boolean'
            ? item.strictVersion
            : item.import !== false && !item.singleton,
        packageName: item.packageName,
        singleton: !!item.singleton,
        eager: !!item.eager,
        issuerLayer: enhanced ? item.issuerLayer : undefined,
        layer: enhanced ? item.layer : undefined,
        request,
        treeShakingMode: item.treeShakingMode,
      };
    },
  );
}

export class ConsumeSharedPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.ConsumeSharedPlugin;
  _options;

  constructor(options: ConsumeSharedPluginOptions) {
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
