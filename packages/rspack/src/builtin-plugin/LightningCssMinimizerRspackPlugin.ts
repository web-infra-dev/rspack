import {
  BuiltinPluginName,
  type RawLightningCssMinimizerOptions,
  type RawLightningCssMinimizerRspackPluginOptions,
} from '@rspack/binding';
import {
  type Drafts,
  type FeatureOptions,
  type NonStandard,
  type PseudoClasses,
  type Targets,
  toFeatures,
} from '../builtin-loader/lightningcss';
import {
  encodeTargets,
  resolveDefaultLightningCssTargets,
} from '../builtin-loader/lightningcss/target';
import type { Compiler } from '../Compiler';
import type { AssetConditions } from '../util/assetCondition';
import { create } from './base';

export type LightningCssMinimizerRspackPluginOptions = {
  test?: AssetConditions;
  include?: AssetConditions;
  exclude?: AssetConditions;
  removeUnusedLocalIdents?: boolean;
  minimizerOptions?: {
    errorRecovery?: boolean;
    targets?: string[] | string | Targets;
    include?: FeatureOptions;
    exclude?: FeatureOptions;
    drafts?: Drafts;
    nonStandard?: NonStandard;
    pseudoClasses?: PseudoClasses;
    unusedSymbols?: string[];
  };
};

export const LightningCssMinimizerRspackPlugin = create(
  BuiltinPluginName.LightningCssMinimizerRspackPlugin,
  function (
    this: Compiler,
    options?: LightningCssMinimizerRspackPluginOptions,
  ): RawLightningCssMinimizerRspackPluginOptions {
    const { include, exclude, nonStandard, pseudoClasses, drafts } =
      options?.minimizerOptions ?? {};
    let targets: RawLightningCssMinimizerOptions['targets'] = [
      'fully supports es6',
    ];
    if (options?.minimizerOptions?.targets) {
      if (typeof options.minimizerOptions.targets === 'string')
        targets = [options.minimizerOptions.targets];
      else if (Array.isArray(options.minimizerOptions.targets))
        targets = options.minimizerOptions.targets;
      else if (typeof options.minimizerOptions.targets === 'object')
        targets = encodeTargets(options.minimizerOptions.targets);
    } else if (this.target.platforms) {
      // Default target derived from rspack target
      targets = resolveDefaultLightningCssTargets(this.target.platforms);
    }
    return {
      test: options?.test,
      include: options?.include,
      exclude: options?.exclude,
      removeUnusedLocalIdents: options?.removeUnusedLocalIdents ?? true,
      minimizerOptions: {
        errorRecovery: options?.minimizerOptions?.errorRecovery ?? true,
        unusedSymbols: options?.minimizerOptions?.unusedSymbols ?? [],
        include: include ? toFeatures(include) : undefined,
        exclude: exclude ? toFeatures(exclude) : undefined,
        targets,
        drafts: drafts
          ? { customMedia: drafts.customMedia ?? false }
          : undefined,
        nonStandard: nonStandard
          ? {
              deepSelectorCombinator:
                nonStandard.deepSelectorCombinator ?? false,
            }
          : undefined,
        pseudoClasses,
      },
    };
  },
);
