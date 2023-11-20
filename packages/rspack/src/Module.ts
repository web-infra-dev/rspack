import {
	JsCodegenerationResult,
	JsCodegenerationResults,
	JsModule
} from "@rspack/binding";

export class Module {
	#inner: JsModule;

	static __from_binding(module: JsModule) {
		return new Module(module);
	}

	constructor(module: JsModule) {
		this.#inner = module;
	}

	get context(): string | undefined {
		return this.#inner.context;
	}

	get resource(): string | undefined {
		return this.#inner.resource;
	}

	identifier(): string {
		return this.#inner.moduleIdentifier;
	}

	nameForCondition(): string | null {
		if (typeof this.#inner.nameForCondition === "string") {
			return this.#inner.nameForCondition;
		} else {
			return null;
		}
	}
}

export class CodeGenerationResult {
	#inner: JsCodegenerationResult;

	constructor(result: JsCodegenerationResult) {
		this.#inner = result;
	}

	get(sourceType: string) {
		return this.#inner.sources[sourceType];
	}
}

export class CodeGenerationResults {
	#inner: JsCodegenerationResults;
	constructor(result: JsCodegenerationResults) {
		this.#inner = result;
	}
}
