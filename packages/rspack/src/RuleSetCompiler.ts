class RuleSetCompiler {
  references: Map<string, any>;
  /**
   * builtin references that should be serializable and passed to Rust.
   */
  builtinReferences: Record<string, string>;
  constructor() {
    this.references = new Map();
    this.builtinReferences = Object.create(null);
  }
}

export { RuleSetCompiler };
