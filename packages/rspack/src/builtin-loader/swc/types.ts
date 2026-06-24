import type {
  Config,
  EnvConfig,
  EsParserConfig,
  JscConfig,
  ModuleConfig,
  ParserConfig,
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

type SwcLoaderCommonOptions = Omit<Config, 'jsc'> & {
  isModule?: boolean | 'unknown';
  /**
   * Collects information from TypeScript's AST for consumption by subsequent Rspack processes,
   * providing better TypeScript development experience and smaller output bundle size.
   */
  collectTypeScriptInfo?: CollectTypeScriptInfoOptions;
  /**
   * Ported from [babel-plugin-import](https://github.com/umijs/babel-plugin-import),
   * used to transform imports for modular component libraries.
   */
  transformImport?: PluginImportOptions;
  /**
   * Experimental features provided by Rspack.
   * @experimental
   */
  rspackExperiments?: {
    /**
     * @deprecated Use top-level `transformImport` instead.
     */
    import?: PluginImportOptions;
    /**
     * Enable React Server Components support.
     */
    reactServerComponents?: boolean | ReactServerComponentsOptions;
  };
};

export type SwcLoaderOptions =
  | (SwcLoaderCommonOptions & {
      /**
       * When set to `"auto"`, `builtin:swc-loader` infers `jsc.parser` from the resource extension.
       * This is useful when one rule needs to handle mixed module types such as `.js`, `.jsx`, `.ts`, and `.tsx`.
       * @default false
       */
      detectSyntax?: false;
      jsc?: JscConfig;
    })
  | (SwcLoaderCommonOptions & {
      detectSyntax: 'auto';
      jsc?: Omit<JscConfig, 'parser'> & {
        // `detectSyntax: 'auto'` allows partial `jsc.parser` options.
        parser?: Partial<ParserConfig>;
      };
    });

export interface ReactServerComponentsOptions {
  /**
   * Whether to disable the compile-time check that reports errors when React
   * client-only APIs (e.g. `useState`, `useEffect`) are imported in server
   * components. Defaults to `false`.
   */
  disableClientApiChecks?: boolean;
}
