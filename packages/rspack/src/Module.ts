import {
	JsCodegenerationResult,
	JsCodegenerationResults,
	JsCreateData,
	JsModule
} from "@rspack/binding";
import { Source } from "webpack-sources";

import { Compilation } from "./Compilation";
import { JsSource } from "./util/source";

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
	createData?: CreateData;
};

export type ContextModuleFactoryBeforeResolveResult =
	| false
	| {
			context: string;
			request?: string;
	  };

export type ContextModuleFactoryAfterResolveResult =
	| false
	| {
			resource: string;
			context: string;
			request: string;
			regExp?: RegExp;
			dependencies: Array<any>;
	  };

export class Module {
	#inner: JsModule;
	#originalSource?: Source;

	context?: Readonly<string>;
	resource?: Readonly<string>;
	request?: Readonly<string>;
	userRequest?: Readonly<string>;
	rawRequest?: Readonly<string>;

	/**
	 * Records the dynamically added fields for Module on the JavaScript side.
	 * These fields are generally used within a plugin, so they do not need to be passed back to the Rust side.
	 * @see {@link Compilation#customModules}
	 */
	buildInfo: Record<string, any>;

	/**
	 * Records the dynamically added fields for Module on the JavaScript side.
	 * These fields are generally used within a plugin, so they do not need to be passed back to the Rust side.
	 * @see {@link Compilation#customModules}
	 */
	buildMeta: Record<string, any>;

	static __from_binding(module: JsModule, compilation?: Compilation) {
		return new Module(module, compilation);
	}

	constructor(module: JsModule, compilation?: Compilation) {
		this.#inner = module;
		this.context = module.context;
		this.resource = module.resource;
		this.request = module.request;
		this.userRequest = module.userRequest;
		this.rawRequest = module.rawRequest;

		const customModule = compilation?.__internal__getCustomModule(
			module.moduleIdentifier
		);
		this.buildInfo = customModule?.buildInfo || {};
		this.buildMeta = customModule?.buildMeta || {};
	}

	originalSource(): Source | null {
		if (this.#originalSource) return this.#originalSource;
		if (this.#inner.originalSource) {
			this.#originalSource = JsSource.__from_binding(
				this.#inner.originalSource
			);
			return this.#originalSource;
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
