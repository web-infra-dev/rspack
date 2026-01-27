import {
  BuiltinPluginName,
  type RawLightningCssMinimizerRspackPluginOptions,
} from '@rspack/binding';

import {
  type Drafts,
  type FeatureOptions,
  type NonStandard,
  type PseudoClasses,
  toFeatures,
} from '../builtin-loader/lightningcss';
import type { AssetConditions } from '../util/assetCondition';
import { create } from './base';

export type LightningCssMinimizerRspackPluginOptions = {
  test?: AssetConditions;
  include?: AssetConditions;
  exclude?: AssetConditions;
  removeUnusedLocalIdents?: boolean;
  minimizerOptions?: {
    errorRecovery?: boolean;
    targets?: string[] | string;
    include?: FeatureOptions;
    exclude?: FeatureOptions;
    /**
     * @deprecated Use `drafts` instead.
     * This will be removed in the next major version.
     */
    draft?: Drafts;
    drafts?: Drafts;
    nonStandard?: NonStandard;
    pseudoClasses?: PseudoClasses;
    unusedSymbols?: string[];
  };
};

export const LightningCssMinimizerRspackPlugin = create(
  BuiltinPluginName.LightningCssMinimizerRspackPlugin,
  (
    options?: LightningCssMinimizerRspackPluginOptions,
  ): RawLightningCssMinimizerRspackPluginOptions => {
    const { include, exclude, draft, nonStandard, pseudoClasses, drafts } =
      options?.minimizerOptions ?? {};
    const targets = options?.minimizerOptions?.targets ?? 'fully supports es6'; // last not support es module chrome version
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
        targets: typeof targets === 'string' ? [targets] : targets,
        draft: draft ? { customMedia: draft.customMedia ?? false } : undefined,
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
