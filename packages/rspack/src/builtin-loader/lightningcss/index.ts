// code modified based on https://github.com/parcel-bundler/lightningcss/blob/34b67a431c043fda5d4979bcdccb3008d082e243/node/browserslistToTargets.js

import type { GetLoaderOptions } from '../../config/adapterRuleUse';
import { defaultTargetsFromRspackTargets, encodeTargets } from './target';

/**
MIT License

Copyright (c) 2021-present Devon Govett

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */

export function toFeatures(featureOptions: FeatureOptions): Features {
  let feature = 0;
  for (const key of Reflect.ownKeys(featureOptions)) {
    if (featureOptions[key as keyof FeatureOptions] !== true) {
      continue;
    }
    switch (key as keyof FeatureOptions) {
      case 'nesting':
        feature |= Features.Nesting;
        break;
      case 'notSelectorList':
        feature |= Features.NotSelectorList;
        break;
      case 'dirSelector':
        feature |= Features.DirSelector;
        break;
      case 'langSelectorList':
        feature |= Features.LangSelectorList;
        break;
      case 'isSelector':
        feature |= Features.IsSelector;
        break;
      case 'textDecorationThicknessPercent':
        feature |= Features.TextDecorationThicknessPercent;
        break;
      case 'mediaIntervalSyntax':
        feature |= Features.MediaIntervalSyntax;
        break;
      case 'mediaRangeSyntax':
        feature |= Features.MediaRangeSyntax;
        break;
      case 'customMediaQueries':
        feature |= Features.CustomMediaQueries;
        break;
      case 'clampFunction':
        feature |= Features.ClampFunction;
        break;
      case 'colorFunction':
        feature |= Features.ColorFunction;
        break;
      case 'oklabColors':
        feature |= Features.OklabColors;
        break;
      case 'labColors':
        feature |= Features.LabColors;
        break;
      case 'p3Colors':
        feature |= Features.P3Colors;
        break;
      case 'hexAlphaColors':
        feature |= Features.HexAlphaColors;
        break;
      case 'spaceSeparatedColorNotation':
        feature |= Features.SpaceSeparatedColorNotation;
        break;
      case 'fontFamilySystemUi':
        feature |= Features.FontFamilySystemUi;
        break;
      case 'doublePositionGradients':
        feature |= Features.DoublePositionGradients;
        break;
      case 'vendorPrefixes':
        feature |= Features.VendorPrefixes;
        break;
      case 'logicalProperties':
        feature |= Features.LogicalProperties;
        break;
      case 'selectors':
        feature |= Features.Selectors;
        break;
      case 'mediaQueries':
        feature |= Features.MediaQueries;
        break;
      case 'color':
        feature |= Features.Color;
        break;
    }
  }
  return feature;
}

export enum Features {
  Empty = /*													*/ 0,
  Nesting = /*												*/ 1 << 0,
  NotSelectorList = /*								*/ 1 << 1,
  DirSelector = /*										*/ 1 << 2,
  LangSelectorList = /*								*/ 1 << 3,
  IsSelector = /*											*/ 1 << 4,
  TextDecorationThicknessPercent = /*	*/ 1 << 5,
  MediaIntervalSyntax = /*						*/ 1 << 6,
  MediaRangeSyntax = /*								*/ 1 << 7,
  CustomMediaQueries = /*							*/ 1 << 8,
  ClampFunction = /*									*/ 1 << 9,
  ColorFunction = /*									*/ 1 << 10,
  OklabColors = /*										*/ 1 << 11,
  LabColors = /* 											*/ 1 << 12,
  P3Colors = /*												*/ 1 << 13,
  HexAlphaColors = /*									*/ 1 << 14,
  SpaceSeparatedColorNotation = /*	 	*/ 1 << 15,
  FontFamilySystemUi = /*							*/ 1 << 16,
  DoublePositionGradients = /*				*/ 1 << 17,
  VendorPrefixes = /*									*/ 1 << 18,
  LogicalProperties = /*							*/ 1 << 19,
  Selectors = Features.Nesting |
    Features.NotSelectorList |
    Features.DirSelector |
    Features.LangSelectorList |
    Features.IsSelector,
  MediaQueries = Features.MediaIntervalSyntax |
    Features.MediaRangeSyntax |
    Features.CustomMediaQueries,
  Color = Features.ColorFunction |
    Features.OklabColors |
    Features.LabColors |
    Features.P3Colors |
    Features.HexAlphaColors |
    Features.SpaceSeparatedColorNotation,
}

export interface Targets {
  android?: string;
  chrome?: string;
  edge?: string;
  firefox?: string;
  ie?: string;
  ios_saf?: string;
  opera?: string;
  safari?: string;
  samsung?: string;
}

export interface Drafts {
  /** Whether to enable @custom-media rules. */
  customMedia?: boolean;
}

export interface NonStandard {
  /** Whether to enable the non-standard >>> and /deep/ selector combinators used by Angular and Vue. */
  deepSelectorCombinator?: boolean;
}

export interface PseudoClasses {
  hover?: string;
  active?: string;
  focus?: string;
  focusVisible?: string;
  focusWithin?: string;
}

export type FeatureOptions = {
  nesting?: boolean;
  notSelectorList?: boolean;
  dirSelector?: boolean;
  langSelectorList?: boolean;
  isSelector?: boolean;
  textDecorationThicknessPercent?: boolean;
  mediaIntervalSyntax?: boolean;
  mediaRangeSyntax?: boolean;
  customMediaQueries?: boolean;
  clampFunction?: boolean;
  colorFunction?: boolean;
  oklabColors?: boolean;
  labColors?: boolean;
  p3Colors?: boolean;
  hexAlphaColors?: boolean;
  spaceSeparatedColorNotation?: boolean;
  fontFamilySystemUi?: boolean;
  doublePositionGradients?: boolean;
  vendorPrefixes?: boolean;
  logicalProperties?: boolean;
  selectors?: boolean;
  mediaQueries?: boolean;
  color?: boolean;
};

export type LoaderOptions = {
  minify?: boolean;
  errorRecovery?: boolean;
  targets?: Targets | string[] | string;
  include?: FeatureOptions;
  exclude?: FeatureOptions;
  drafts?: Drafts;
  nonStandard?: NonStandard;
  pseudoClasses?: PseudoClasses;
  unusedSymbols?: string[];
};

export const getLightningcssLoaderOptions: GetLoaderOptions = (
  o,
  composeOptions,
) => {
  const options = o ?? {};
  if (typeof options === 'object') {
    if (typeof options.targets === 'string') {
      options.targets = [options.targets];
    } else if (
      typeof options.targets === 'object' &&
      !Array.isArray(options.targets)
    ) {
      options.targets = encodeTargets(options.targets);
    } else if (
      options.targets === undefined &&
      composeOptions.compiler.target?.targets
    ) {
      // Default target derived from rspack target
      options.targets = defaultTargetsFromRspackTargets(
        composeOptions.compiler.target.targets,
      );
    }

    if (options.include && typeof options.include === 'object') {
      options.include = toFeatures(
        options.include as unknown as FeatureOptions,
      );
    }

    if (options.exclude && typeof options.exclude === 'object') {
      options.exclude = toFeatures(
        options.exclude as unknown as FeatureOptions,
      );
    }
  }

  return options;
};
