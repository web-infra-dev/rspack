import type { Compiler } from '../Compiler';
import { parseOptions } from '../container/options';
import { ConsumeSharedPlugin } from './ConsumeSharedPlugin';
import { ProvideSharedPlugin } from './ProvideSharedPlugin';
import {
  isRequiredVersion,
  resolveShareKey,
  resolveShareRequest,
} from './utils';

export type ShareScope = string | string[];

export function validateShareScope(
  shareScope: ShareScope,
  enhanced: boolean,
  pluginName: string,
): void {
  if (Array.isArray(shareScope) && shareScope.length > 1 && !enhanced) {
    throw new Error(
      `[${pluginName}] Multiple share scopes are only supported in enhanced mode. Got: ${JSON.stringify(shareScope)}`,
    );
  }
}

export type SharePluginOptions = {
  shareScope?: ShareScope;
  shared: Shared;
  enhanced: boolean;
};
export type Shared = (SharedItem | SharedObject)[] | SharedObject;
export type SharedItem = string;
export type SharedObject = {
  [k: string]: SharedConfig | SharedItem;
};
export type TreeShakingConfig = {
  usedExports?: string[];
  mode?: 'server-calc' | 'runtime-infer';
  filename?: string;
};

export type SharedConfig = {
  eager?: boolean;
  import?: false | SharedItem;
  issuerLayer?: string;
  layer?: string;
  packageName?: string;
  request?: string;
  requiredVersion?: false | string;
  shareKey?: string;
  shareScope?: ShareScope;
  singleton?: boolean;
  strictVersion?: boolean;
  version?: false | string;
  treeShaking?: TreeShakingConfig;
};

export type NormalizedSharedOptions = [string, SharedConfig][];

export function normalizeSharedOptions(
  shared: Shared,
): NormalizedSharedOptions {
  return parseOptions(
    shared,
    (item, key) => {
      if (typeof item !== 'string')
        throw new Error('Unexpected array in shared');
      const config: SharedConfig =
        item === key || !isRequiredVersion(item)
          ? {
              import: item,
            }
          : {
              import: key,
              requiredVersion: item,
            };
      return config;
    },
    (item) => item,
  );
}

export function createProvideShareOptions(
  normalizedSharedOptions: NormalizedSharedOptions,
  enhanced: boolean,
) {
  return normalizedSharedOptions
    .filter(([, options]) => options.import !== false)
    .map(([key, options]) => ({
      [options.import || key]: {
        shareKey: resolveShareKey(options.shareKey, key),
        shareScope: options.shareScope,
        version: options.version,
        eager: options.eager,
        singleton: options.singleton,
        requiredVersion: options.requiredVersion,
        strictVersion: options.strictVersion,
        layer: enhanced ? options.layer : undefined,
        request: resolveShareRequest(options.request, options.import || key),
        treeShakingMode: options.treeShaking?.mode,
      },
    }));
}

export function createConsumeShareOptions(
  normalizedSharedOptions: NormalizedSharedOptions,
  enhanced: boolean,
) {
  return normalizedSharedOptions.map(([key, options]) => ({
    [key]: {
      import: options.import,
      shareKey: resolveShareKey(options.shareKey, key),
      shareScope: options.shareScope,
      requiredVersion: options.requiredVersion,
      strictVersion: options.strictVersion,
      singleton: options.singleton,
      packageName: options.packageName,
      eager: options.eager,
      issuerLayer: enhanced ? options.issuerLayer : undefined,
      layer: enhanced ? options.layer : undefined,
      request: resolveShareRequest(options.request, key),
      treeShakingMode: options.treeShaking?.mode,
    },
  }));
}
export class SharePlugin {
  _shareScope;
  _consumes;
  _provides;
  _enhanced;
  _sharedOptions;

  constructor(options: SharePluginOptions) {
    const enhanced = options.enhanced ?? false;
    const sharedOptions = normalizeSharedOptions(options.shared);
    if (options.shareScope) {
      validateShareScope(options.shareScope, enhanced, 'SharePlugin');
    }
    for (const [, config] of sharedOptions) {
      if (config.shareScope) {
        validateShareScope(config.shareScope, enhanced, 'SharePlugin');
      }
    }
    const consumes = createConsumeShareOptions(sharedOptions, enhanced);
    const provides = createProvideShareOptions(sharedOptions, enhanced);
    this._shareScope = options.shareScope;
    this._consumes = consumes;
    this._provides = provides;
    this._enhanced = enhanced;
    this._sharedOptions = sharedOptions;
  }

  apply(compiler: Compiler) {
    new ConsumeSharedPlugin({
      shareScope: this._shareScope,
      consumes: this._consumes,
      enhanced: this._enhanced,
    }).apply(compiler);
    new ProvideSharedPlugin({
      shareScope: this._shareScope,
      provides: this._provides,
      enhanced: this._enhanced,
    }).apply(compiler);
  }
}
