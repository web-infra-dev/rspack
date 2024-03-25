import {
	JsCodegenerationResult,
	JsCodegenerationResults,
	JsCreateData,
	JsFactoryMeta,
	JsModule
} from "@rspack/binding";
import { Source } from "webpack-sources";
import { createSourceFromRaw } from "./util/createSource";

export type ResourceData = {
	resource: string;
	path: string;
	query?: string;
	fragment?: string;
};
export type ResourceDataWithData = ResourceData & {
	data?: Record<string, any>;
};
export type CreateData = Partial<JsCreateData>;
export type ResolveData = {
	context: string;
	request: string;
	fileDependencies: string[];
	missingDependencies: string[];
	contextDependencies: string[];
	factoryMeta?: JsFactoryMeta;
	createData?: CreateData;
};

export class Module {
	#inner: JsModule;
	_originalSource?: Source;

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

	get originalSource(): Source | null {
		if (this._originalSource) return this._originalSource;
		if (this.#inner.originalSource) {
			this._originalSource = createSourceFromRaw(this.#inner.originalSource);
			return this._originalSource;
		} else {
			return null;
		}
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
