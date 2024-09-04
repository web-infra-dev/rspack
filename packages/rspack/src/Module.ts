import type {
	JsCodegenerationResult,
	JsCreateData,
	JsFactoryMeta,
	JsModule,
	ModuleDTO
} from "@rspack/binding";
import type { Source } from "webpack-sources";

import type { Compilation } from "./Compilation";
import { DependenciesBlock } from "./DependenciesBlock";
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
export type ContextInfo = {
	issuer: string;
};
export type ResolveData = {
	contextInfo: ContextInfo;
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
	#inner: JsModule | ModuleDTO;
	#originalSource?: Source;

	context?: Readonly<string>;
	resource?: Readonly<string>;
	request?: Readonly<string>;
	userRequest?: Readonly<string>;
	rawRequest?: Readonly<string>;
	type: string;
	layer: null | string;

	factoryMeta?: Readonly<JsFactoryMeta>;
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

	static __from_binding(
		module: JsModule | ModuleDTO,
		compilation?: Compilation
	) {
		return new Module(module, compilation);
	}

	constructor(module: JsModule | ModuleDTO, compilation?: Compilation) {
		this.#inner = module;
		this.type = module.type;
		this.layer = module.layer ?? null;
		this.context = module.context;
		this.resource = module.resource;
		this.request = module.request;
		this.userRequest = module.userRequest;
		this.rawRequest = module.rawRequest;

		this.factoryMeta = module.factoryMeta;
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
		}
		return null;
	}

	identifier(): string {
		return this.#inner.moduleIdentifier;
	}

	nameForCondition(): string | null {
		if (typeof this.#inner.nameForCondition === "string") {
			return this.#inner.nameForCondition;
		}
		return null;
	}

	get blocks(): DependenciesBlock[] {
		if ("blocks" in this.#inner) {
			return this.#inner.blocks.map(b => new DependenciesBlock(b));
		}
		return [];
	}

	size(type?: string): number {
		if ("size" in this.#inner) {
			return this.#inner.size(type);
		}
		return 0;
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

export class CodeGenerationResults {}
