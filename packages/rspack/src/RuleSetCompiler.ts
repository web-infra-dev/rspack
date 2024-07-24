class RuleSetCompiler {
	references: Map<string, any>;
	/**
	 * builtin references that should be serializable and passed to Rust.
	 */
	builtinReferences: Map<string, any>;
	constructor() {
		this.references = new Map();
		this.builtinReferences = new Map();
	}
}

export { RuleSetCompiler };
