import {
  BuiltinPluginName,
  type Chunk,
  type RawBannerPluginOptions,
} from '@rspack/binding';

import { create } from './base';

export type Rule = string | RegExp;

export type Rules = Rule[] | Rule;

export type BannerFunction = (args: {
  hash: string;
  chunk: Chunk;
  filename: string;
}) => string;

export type BannerContent = string | BannerFunction;

export type BannerPluginOptions = {
  /** Specifies the banner, it will be wrapped in a comment. */
  banner: BannerContent;

  /** If true, the banner will only be added to the entry chunks. */
  entryOnly?: boolean;

  /** Exclude all modules matching any of these conditions. */
  exclude?: Rules;

  /** Include all modules matching any of these conditions. */
  include?: Rules;

  /** If true, banner will not be wrapped in a comment. */
  raw?: boolean;

  /** If true, banner will be placed at the end of the output. */
  footer?: boolean;

  /**
   * The stage of the compilation in which the banner should be injected.
   * @default PROCESS_ASSETS_STAGE_ADDITIONS (-100)
   */
  stage?: number;

  /** Include all modules that pass test assertion. */
  test?: Rules;
};

export type BannerPluginArgument = BannerContent | BannerPluginOptions;

export const BannerPlugin = create(
  BuiltinPluginName.BannerPlugin,
  (args: BannerPluginArgument): RawBannerPluginOptions => {
    if (typeof args === 'string' || typeof args === 'function') {
      return {
        banner: args,
      };
    }

    return {
      banner: args.banner,
      entryOnly: args.entryOnly,
      footer: args.footer,
      raw: args.raw,
      test: args.test,
      stage: args.stage,
      include: args.include,
      exclude: args.exclude,
    };
  },
);
