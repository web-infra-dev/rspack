import type {
	JsCodegenerationResult,
	JsContextModuleFactoryAfterResolveData,
	JsContextModuleFactoryBeforeResolveData,
	JsCreateData,
	JsFactoryMeta
} from "@rspack/binding";
import type { Source } from "webpack-sources";

import { JsModule } from "@rspack/binding";
import { DependenciesBlock } from "./DependenciesBlock";
import { Dependency, bindingDependencyFactory } from "./Dependency";
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
	issuerLayer?: string | null;
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

export interface LibIdentOptions {
	/**
	 * absolute context path to which lib ident is relative to
	 */
	context: string;
}

export class ContextModuleFactoryBeforeResolveData {
	#inner: JsContextModuleFactoryBeforeResolveData;

	declare context: string;
	declare request: string;
	declare regExp: RegExp | undefined;
	declare recursive: boolean;

	static __from_binding(binding: JsContextModuleFactoryBeforeResolveData) {
		return new ContextModuleFactoryBeforeResolveData(binding);
	}

	static __to_binding(
		data: ContextModuleFactoryBeforeResolveData
	): JsContextModuleFactoryBeforeResolveData {
		return data.#inner;
	}

	private constructor(binding: JsContextModuleFactoryBeforeResolveData) {
		this.#inner = binding;

		Object.defineProperties(this, {
			context: {
				enumerable: true,
				get(): string {
					return binding.context;
				},
				set(val: string) {
					binding.context = val;
				}
			},
			request: {
				enumerable: true,
				get(): string {
					return binding.request;
				},
				set(val: string) {
					binding.request = val;
				}
			},
			regExp: {
				enumerable: true,
				get(): RegExp | undefined {
					return binding.regExp;
				},
				set(val: RegExp | undefined) {
					binding.regExp = val;
				}
			},
			recursive: {
				enumerable: true,
				get(this: ContextModuleFactoryAfterResolveData): boolean {
					return binding.recursive;
				},
				set(val: boolean) {
					binding.recursive = val;
				}
			}
		});
	}
}

export type ContextModuleFactoryBeforeResolveResult =
	| false
	| ContextModuleFactoryBeforeResolveData;

export class ContextModuleFactoryAfterResolveData {
	#inner: JsContextModuleFactoryAfterResolveData;

	declare resource: number;
	declare context: string;
	declare request: string;
	declare regExp: RegExp | undefined;
	declare recursive: boolean;
	declare readonly dependencies: Dependency[];

	static __from_binding(binding: JsContextModuleFactoryAfterResolveData) {
		return new ContextModuleFactoryAfterResolveData(binding);
	}

	static __to_binding(
		data: ContextModuleFactoryAfterResolveData
	): JsContextModuleFactoryAfterResolveData {
		return data.#inner;
	}

	private constructor(binding: JsContextModuleFactoryAfterResolveData) {
		this.#inner = binding;

		Object.defineProperties(this, {
			resource: {
				enumerable: true,
				get(): string {
					return binding.resource;
				},
				set(val: string) {
					binding.resource = val;
				}
			},
			context: {
				enumerable: true,
				get(): string {
					return binding.context;
				},
				set(val: string) {
					binding.context = val;
				}
			},
			request: {
				enumerable: true,
				get(): string {
					return binding.request;
				},
				set(val: string) {
					binding.request = val;
				}
			},
			regExp: {
				enumerable: true,
				get(): RegExp | undefined {
					return binding.regExp;
				},
				set(val: RegExp | undefined) {
					binding.regExp = val;
				}
			},
			recursive: {
				enumerable: true,
				get(): boolean {
					return binding.recursive;
				},
				set(val: boolean) {
					binding.recursive = val;
				}
			},
			dependencies: {
				enumerable: true,
				get(): Dependency[] {
					return binding.dependencies.map(dep =>
						bindingDependencyFactory.create(Dependency, dep)
					);
				}
			}
		});
	}
}

export type ContextModuleFactoryAfterResolveResult =
	| false
	| ContextModuleFactoryAfterResolveData;

const MODULE_MAPPINGS = new WeakMap<JsModule, Module>();

export class Module {
	#inner: JsModule;

	#identifier: string | undefined;
	#constructorName: string | undefined;
	#type: string | undefined;
	#layer: string | undefined | null;
	#context: string | undefined | null;
	#resource: string | undefined | null;
	#request: string | undefined | null;
	#rawRequest: string | undefined | null;
	#resourceResolveData: ResolveData | undefined | null;
	#matchResource: string | undefined | null;
	#modules: Module[] | undefined | null;

	declare readonly context: string | null;
	declare readonly resource: string | null;
	declare readonly request: string | null;
	declare userRequest?: string;
	declare readonly rawRequest: string | null;
	declare readonly type: string;
	declare readonly layer: string | null;
	declare readonly factoryMeta?: JsFactoryMeta;
	declare readonly modules: Module[] | undefined;
	declare readonly blocks: DependenciesBlock[];
	declare readonly dependencies: Dependency[];
	declare readonly useSourceMap: boolean;

	/**
	 * Records the dynamically added fields for Module on the JavaScript side.
	 * These fields are generally used within a plugin, so they do not need to be passed back to the Rust side.
	 */
	buildInfo: Record<string, any>;

	/**
	 * Records the dynamically added fields for Module on the JavaScript side.
	 * These fields are generally used within a plugin, so they do not need to be passed back to the Rust side.
	 * @see {@link Compilation#customModules}
	 */
	buildMeta: Record<string, any>;

	static __from_binding(binding: JsModule) {
		let module = MODULE_MAPPINGS.get(binding);
		if (module) {
			return module;
		}
		module = new Module(binding);
		MODULE_MAPPINGS.set(binding, module);
		return module;
	}

	static __to_binding(module: Module): JsModule {
		return module.#inner;
	}

	constructor(module: JsModule) {
		this.#inner = module;

		this.buildInfo = module.buildInfo;
		this.buildMeta = {};

		Object.defineProperties(this, {
			constructorName: {
				enumerable: true,
				get: (): string => {
					if (this.#constructorName === undefined) {
						this.#constructorName = module.constructorName;
					}
					return this.#constructorName;
				}
			},
			type: {
				enumerable: true,
				get: (): string => {
					if (this.#type === undefined) {
						this.#type = module.type;
					}
					return this.#type;
				}
			},
			layer: {
				enumerable: true,
				get: (): string | null => {
					if (this.#layer === undefined) {
						this.#layer = module.layer;
					}
					return this.#layer;
				}
			},
			context: {
				enumerable: true,
				get: (): string | null => {
					if (this.#context === undefined) {
						this.#context = module.context;
					}
					return this.#context;
				}
			},
			resource: {
				enumerable: true,
				get: (): string | null => {
					if (this.#resource === undefined) {
						this.#resource = module.resource;
					}
					return this.#resource;
				}
			},
			request: {
				enumerable: true,
				get: (): string | null => {
					if (this.#request === undefined) {
						this.#request = module.request;
					}
					return this.#request;
				}
			},
			userRequest: {
				enumerable: true,
				get(): string | undefined {
					return module.userRequest;
				},
				set(val: string) {
					module.userRequest = val;
				}
			},
			rawRequest: {
				enumerable: true,
				get: (): string | null => {
					if (this.#rawRequest === undefined) {
						this.#rawRequest = module.rawRequest;
					}
					return this.#rawRequest;
				}
			},
			factoryMeta: {
				enumerable: true,
				get(): JsFactoryMeta | undefined | undefined {
					return module.factoryMeta;
				}
			},
			modules: {
				enumerable: true,
				get: (): Module[] | null => {
					if (module instanceof JsModule) {
						if (this.#modules !== undefined) {
							return this.#modules;
						}
						this.#modules = module.modules
							? module.modules.map(m => Module.__from_binding(m))
							: null;
						return this.#modules;
					}
					return null;
				}
			},
			blocks: {
				enumerable: true,
				get(): DependenciesBlock[] {
					return module.blocks.map(b => DependenciesBlock.__from_binding(b));
				}
			},
			dependencies: {
				enumerable: true,
				get(): Dependency[] {
					if ("dependencies" in module) {
						return module.dependencies.map(d =>
							bindingDependencyFactory.create(Dependency, d)
						);
					}
					return [];
				}
			},
			useSourceMap: {
				enumerable: true,
				get(): boolean {
					return module.useSourceMap;
				}
			},
			resourceResolveData: {
				enumerable: true,
				get: (): ResolveData | null => {
					if (this.#resourceResolveData === undefined) {
						this.#resourceResolveData = module.resourceResolveData as any;
					}
					return this.#resourceResolveData!;
				}
			},
			matchResource: {
				enumerable: true,
				get: (): string | null => {
					if (this.#matchResource === undefined) {
						this.#matchResource = module.matchResource;
					}
					return this.#matchResource;
				}
			}
		});
	}

	originalSource(): Source | null {
		if (this.#inner.originalSource) {
			return JsSource.__from_binding(this.#inner.originalSource);
		}
		return null;
	}

	identifier(): string {
		if (this.#identifier === undefined) {
			this.#identifier = this.#inner.moduleIdentifier;
		}
		return this.#identifier;
	}

	nameForCondition(): string | null {
		if (typeof this.#inner.nameForCondition === "string") {
			return this.#inner.nameForCondition;
		}
		return null;
	}

	size(type?: string): number {
		if ("size" in this.#inner) {
			return this.#inner.size(type);
		}
		return 0;
	}

	libIdent(options: LibIdentOptions): string | null {
		return this.#inner.libIdent(options);
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
