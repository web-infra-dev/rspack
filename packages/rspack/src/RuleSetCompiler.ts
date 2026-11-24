import type { RuleSetLoaderWithOptions } from './config/types';

export interface LoaderMetadata {
  cache: boolean;
  optionsCacheKey: string;
  codeFilter?: string | RegExp;
  parallelism?: RuleSetLoaderWithOptions['parallel'];
}

class RuleSetCompiler {
  references: Map<string, any>;
  /**
   * builtin references that should be serializable and passed to Rust.
   */
  builtinReferences: Map<string, any>;
  loaderMetadata: Map<string, LoaderMetadata>;
  constructor() {
    this.references = new Map();
    this.builtinReferences = new Map();
    this.loaderMetadata = new Map();
  }
}

export { RuleSetCompiler };
