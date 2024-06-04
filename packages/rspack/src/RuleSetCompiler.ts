class RuleSetCompiler {
	public references: Map<string, any>;
	// builtin references that should be serializable
	public builtinReferences: Map<string, any>;
	constructor() {
		this.references = new Map();
		this.builtinReferences = new Map();
	}
}

export { RuleSetCompiler };
