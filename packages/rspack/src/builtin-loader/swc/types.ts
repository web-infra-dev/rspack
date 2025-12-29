import type {
  Config,
  EnvConfig,
  EsParserConfig,
  JscConfig,
  ModuleConfig,
  ParserConfig,
  TerserEcmaVersion,
  TransformConfig,
  TsParserConfig,
} from '@swc/types';
import type { CollectTypeScriptInfoOptions } from './collectTypeScriptInfo';
import type { PluginImportOptions } from './pluginImport';
export type SwcLoaderEnvConfig = EnvConfig;
export type SwcLoaderJscConfig = JscConfig;
export type SwcLoaderModuleConfig = ModuleConfig;
export type SwcLoaderParserConfig = ParserConfig;
export type SwcLoaderEsParserConfig = EsParserConfig;
export type SwcLoaderTsParserConfig = TsParserConfig;
export type SwcLoaderTransformConfig = TransformConfig;
export type SwcLoaderOptions = Config & {
  isModule?: boolean | 'unknown';
  /**
   * Collects information from TypeScript's AST for consumption by subsequent Rspack processes,
   * providing better TypeScript development experience and smaller output bundle size.
   */
  collectTypeScriptInfo?: CollectTypeScriptInfoOptions;
  /**
   * Experimental features provided by Rspack.
   * @experimental
   */
  rspackExperiments?: {
    import?: PluginImportOptions;
    /**
     * @deprecated Use top-level `collectTypeScriptInfo` instead.
     * Collects information from TypeScript's AST for consumption by subsequent Rspack processes,
     * providing better TypeScript development experience and smaller output bundle size.
     */
    collectTypeScriptInfo?: CollectTypeScriptInfoOptions;
  };
};

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
