import {
  BuiltinPluginName,
  type RawExtractComments,
  type RawSwcJsMinimizerRspackPluginOptions,
} from '@rspack/binding';

import type { Compiler } from '../Compiler';
import type { LiteralUnion } from '../config';
import type { AssetConditions } from '../util/assetCondition';
import { create } from './base';

type ExtractCommentsCondition = boolean | RegExp;
type ExtractCommentsBanner = string | boolean;
// type ExtractFilename = string;
type ExtractCommentsObject = {
  condition?: ExtractCommentsCondition | undefined;
  banner?: ExtractCommentsBanner | undefined;
  // filename?: ExtractFilename | undefined
};
type ExtractCommentsOptions = ExtractCommentsCondition | ExtractCommentsObject;

export type SwcJsMinimizerRspackPluginOptions = {
  test?: AssetConditions;
  exclude?: AssetConditions;
  include?: AssetConditions;
  extractComments?: ExtractCommentsOptions | undefined;
  minimizerOptions?: {
    minify?: boolean;
    ecma?: TerserEcmaVersion;
    compress?: TerserCompressOptions | boolean;
    mangle?: TerserMangleOptions | boolean;
    format?: JsFormatOptions & ToSnakeCaseProperties<JsFormatOptions>;
    module?: boolean;
  };
};

/**
 * @example ToSnakeCase<'indentLevel'> == 'indent_level'
 */
type ToSnakeCase<T extends string> = T extends `${infer A}${infer B}`
  ? `${A extends Lowercase<A> ? A : `_${Lowercase<A>}`}${ToSnakeCase<B>}`
  : T;
/**
 * @example ToSnakeCaseProperties\<{indentLevel: 3\}\> == {indent_level: 3\}
 */
type ToSnakeCaseProperties<T> = {
  [K in keyof T as K extends string ? ToSnakeCase<K> : K]: T[K];
};

export interface JsFormatOptions {
  /**
   * Currently noop.
   * @default false
   * @alias ascii_only
   */
  asciiOnly?: boolean;
  /**
   * Currently noop.
   * @default false
   */
  beautify?: boolean;
  /**
   * Currently noop.
   * @default false
   */
  braces?: boolean;
  /**
   * - `false`: removes all comments
   * - `'some'`: preserves some comments
   * - `'all'`: preserves all comments
   * @default false
   */
  comments?: false | 'some' | 'all';
  /**
   * Currently noop.
   * @default 5
   */
  ecma?: TerserEcmaVersion;
  /**
   * Currently noop.
   * @alias indent_level
   */
  indentLevel?: number;
  /**
   * Currently noop.
   * @alias indent_start
   */
  indentStart?: number;
  inlineScript?: number;
  /**
   * Currently noop.
   * @alias keep_numbers
   */
  keepNumbers?: number;
  /**
   * Currently noop.
   * @alias keep_quoted_props
   */
  keepQuotedProps?: boolean;
  /**
   * Currently noop.
   * @alias max_line_len
   */
  maxLineLen?: number | false;
  /**
   * When passed it must be a string and it will be prepended to the output literally.
   * The source map will adjust for this text. Can be used to insert a comment containing licensing information.
   */
  preamble?: string;
  /**
   * Currently noop.
   * @alias quote_keys
   */
  quoteKeys?: boolean;
  /**
   * Currently noop.
   * @alias quote_style
   */
  quoteStyle?: boolean;
  /**
   * Currently noop.
   * @alias preserve_annotations
   */
  preserveAnnotations?: boolean;
  /**
   * Currently noop.
   */
  safari10?: boolean;
  /**
   * Currently noop.
   */
  semicolons?: boolean;
  /**
   * Currently noop.
   */
  shebang?: boolean;
  /**
   * Currently noop.
   */
  webkit?: boolean;
  /**
   * Currently noop.
   * @alias wrap_iife
   */
  wrapIife?: boolean;
  /**
   * Currently noop.
   * @alias wrap_func_args
   */
  wrapFuncArgs?: boolean;
}

export type TerserEcmaVersion = LiteralUnion<5 | 2015 | 2016, number> | string;
export interface TerserCompressOptions {
  arguments?: boolean;
  arrows?: boolean;
  booleans?: boolean;
  booleans_as_integers?: boolean;
  collapse_vars?: boolean;
  comparisons?: boolean;
  computed_props?: boolean;
  conditionals?: boolean;
  dead_code?: boolean;
  defaults?: boolean;
  directives?: boolean;
  drop_console?: boolean;
  drop_debugger?: boolean;
  ecma?: TerserEcmaVersion;
  evaluate?: boolean;
  expression?: boolean;
  global_defs?: any;
  hoist_funs?: boolean;
  hoist_props?: boolean;
  hoist_vars?: boolean;
  ie8?: boolean;
  if_return?: boolean;
  inline?: 0 | 1 | 2 | 3;
  join_vars?: boolean;
  keep_classnames?: boolean;
  keep_fargs?: boolean;
  keep_fnames?: boolean;
  keep_infinity?: boolean;
  loops?: boolean;
  negate_iife?: boolean;
  passes?: number;
  properties?: boolean;
  pure_getters?: any;
  pure_funcs?: string[];
  reduce_funcs?: boolean;
  reduce_vars?: boolean;
  sequences?: any;
  side_effects?: boolean;
  switches?: boolean;
  top_retain?: any;
  toplevel?: any;
  typeofs?: boolean;
  unsafe?: boolean;
  unsafe_passes?: boolean;
  unsafe_arrows?: boolean;
  unsafe_comps?: boolean;
  unsafe_function?: boolean;
  unsafe_math?: boolean;
  unsafe_symbols?: boolean;
  unsafe_methods?: boolean;
  unsafe_proto?: boolean;
  unsafe_regexp?: boolean;
  unsafe_undefined?: boolean;
  unused?: boolean;
  const_to_let?: boolean;
  module?: boolean;
}
export interface TerserMangleOptions {
  props?: TerserManglePropertiesOptions;
  toplevel?: boolean;
  keep_classnames?: boolean;
  keep_fnames?: boolean;
  keep_private_props?: boolean;
  ie8?: boolean;
  safari10?: boolean;
  reserved?: string[];
}
export type TerserManglePropertiesOptions = {};

function isObject(value: any): value is Object {
  const type = typeof value;

  return value != null && (type === 'object' || type === 'function');
}

function getRawExtractCommentsOptions(
  extractComments?: ExtractCommentsOptions,
): RawExtractComments | undefined {
  const conditionStr = (condition?: ExtractCommentsCondition): string => {
    if (typeof condition === 'undefined' || condition === true) {
      // copied from terser-webpack-plugin
      return '@preserve|@lic|@cc_on|^\\**!';
    }
    if (condition === false) {
      throw Error('unreachable');
    }
    // FIXME: flags
    return condition.source;
  };
  if (typeof extractComments === 'boolean') {
    if (!extractComments) {
      return undefined;
    }
    const res = {
      condition: conditionStr(extractComments),
    };
    return res;
  }
  if (extractComments instanceof RegExp) {
    const res = {
      condition: extractComments.source,
    };
    return res;
  }
  if (isObject(extractComments)) {
    if (extractComments.condition === false) {
      return undefined;
    }
    const res = {
      condition: conditionStr(extractComments.condition),
      banner: extractComments.banner,
    };
    return res;
  }
  return undefined;
}

export const SwcJsMinimizerRspackPlugin = create(
  BuiltinPluginName.SwcJsMinimizerRspackPlugin,
  function (
    this: Compiler,
    options?: SwcJsMinimizerRspackPluginOptions,
  ): RawSwcJsMinimizerRspackPluginOptions {
    let compress = options?.minimizerOptions?.compress ?? true;
    const mangle = options?.minimizerOptions?.mangle ?? true;
    const ecma =
      options?.minimizerOptions?.ecma ??
      // Default target derived from rspack target
      this.target?.esVersion ??
      5;
    const format = {
      comments: false, // terser and swc use different default value: 'some'
      ...options?.minimizerOptions?.format,
    };

    // terser defaults to 1, SWC defaults to 3
    // Rspack uses 2 to balance the build performance and the bundle size
    if (compress && typeof compress === 'object') {
      compress = {
        passes: 2,
        ...compress,
      };
    } else if (compress) {
      compress = {
        passes: 2,
      };
    }

    return {
      test: options?.test,
      include: options?.include,
      exclude: options?.exclude,
      extractComments: getRawExtractCommentsOptions(options?.extractComments),
      minimizerOptions: {
        compress,
        mangle,
        ecma,
        format,
        minify: options?.minimizerOptions?.minify,
        module: options?.minimizerOptions?.module,
      },
    };
  },
  'compilation',
);
