import type { GetLoaderOptions } from '../../config/adapterRuleUse';

export interface LessOptions {
  /** Defaults to the resource path. Used for imports, diagnostics, and source maps. */
  filename?: string;
  compress?: boolean;
  strictUnits?: boolean;
  math?: 'always' | 'parens-division' | 'parens' | 'strict' | 0 | 1 | 2;
  /** Legacy alias that overrides `math` with `'parens'` when true. */
  strictMath?: boolean;
  rewriteUrls?: 'off' | 'local' | 'all' | false | 0 | 1 | 2;
  /** Defaults to true. Set false to configure `rewriteUrls` independently. */
  relativeUrls?: boolean;
  rootpath?: string;
  urlArgs?: string;
  /** Additional import directories, relative to Rspack's context. */
  paths?: string | string[];
  globalVars?: Record<string, string | number>;
  modifyVars?: Record<string, string | number>;
  strictImports?: boolean;
  /** JavaScript evaluation is not supported by less-rs. */
  javascriptEnabled?: boolean;
  dumpLineNumbers?: 'comments' | 'mediaquery' | 'all';
  banner?: string;
  /** Overridden when the loader's `sourceMap` setting is enabled. */
  sourceMap?:
    | boolean
    | {
        sourceMapURL?: string;
        sourceMapBasepath?: string;
        sourceMapRootpath?: string;
        sourceMapFilename?: string;
        sourceMapOutputFilename?: string;
        sourceMapInputFilename?: string;
        sourceMapFullFilename?: string;
        sourceMapFileInline?: boolean;
        outputSourceFiles?: boolean;
        disableSourcemapAnnotation?: boolean;
      };
  processImports?: boolean;
  quiet?: boolean;
  quietDeprecations?: boolean;
  /** HTTP imports are not supported by the builtin loader's input file system. */
  insecure?: boolean;
  disablePluginRule?: boolean;
}

export interface LessLoaderOptions {
  /** Resolve imports with Rspack after Less lookup (default true). "only" skips Less lookup. */
  webpackImporter?: boolean | 'only';
  /** Defaults to the source map setting derived from `devtool`. */
  sourceMap?: boolean;
  /** Less code prepended to the resource, followed by a newline. */
  additionalData?: string;
  lessOptions?: LessOptions;
}

export const getLessLoaderOptions: GetLoaderOptions = (options) => {
  if (options && typeof options === 'object') {
    for (const key of ['additionalData', 'lessOptions'] as const) {
      if (typeof options[key] === 'function') {
        throw new Error(
          `builtin:less-loader does not support a function for ${key}.`,
        );
      }
    }
  }
  return options;
};
