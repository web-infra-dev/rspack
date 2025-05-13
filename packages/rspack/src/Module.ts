import * as binding from "@rspack/binding";
import type { Source } from "webpack-sources";
import type { ResourceData } from "./Resolver";
import { JsSource } from "./util/source";

export type ResourceDataWithData = ResourceData & {
	data?: Record<string, any>;
};
export type CreateData = Partial<binding.JsCreateData>;
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

export class ContextModuleFactoryBeforeResolveData {
	#inner: binding.JsContextModuleFactoryBeforeResolveData;

	declare context: string;
	declare request: string;
	declare regExp: RegExp | undefined;
	declare recursive: boolean;

	static __from_binding(
		binding: binding.JsContextModuleFactoryBeforeResolveData
	) {
		return new ContextModuleFactoryBeforeResolveData(binding);
	}

	static __to_binding(
		data: ContextModuleFactoryBeforeResolveData
	): binding.JsContextModuleFactoryBeforeResolveData {
		return data.#inner;
	}

	private constructor(
		binding: binding.JsContextModuleFactoryBeforeResolveData
	) {
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
	#inner: binding.JsContextModuleFactoryAfterResolveData;

	declare resource: number;
	declare context: string;
	declare request: string;
	declare regExp: RegExp | undefined;
	declare recursive: boolean;
	declare readonly dependencies: binding.Dependency[];

	static __from_binding(
		binding: binding.JsContextModuleFactoryAfterResolveData
	) {
		return new ContextModuleFactoryAfterResolveData(binding);
	}

	static __to_binding(
		data: ContextModuleFactoryAfterResolveData
	): binding.JsContextModuleFactoryAfterResolveData {
		return data.#inner;
	}

	private constructor(binding: binding.JsContextModuleFactoryAfterResolveData) {
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
				get(): binding.Dependency[] {
					return binding.dependencies;
				}
			}
		});
	}
}

export type ContextModuleFactoryAfterResolveResult =
	| false
	| ContextModuleFactoryAfterResolveData;

const $assets: unique symbol = Symbol("assets");

declare module "@rspack/binding" {
	interface Assets {
		[$assets]: Record<string, Source>;
	}

	interface BuildInfo {
		assets: Record<string, Source>;
	}
}

Object.defineProperty(binding.BuildInfo.prototype, "assets", {
	enumerable: true,
	configurable: true,
	get(this: binding.BuildInfo): Record<string, Source> {
		if (this._assets[$assets]) {
			return this._assets[$assets];
		}
		const assets = new Proxy(Object.create(null), {
			ownKeys: () => {
				return this._assets.keys();
			},
			getOwnPropertyDescriptor() {
				return {
					enumerable: true,
					configurable: true
				};
			}
		}) as Record<string, Source>;
		Object.defineProperty(this._assets, $assets, {
			enumerable: false,
			configurable: true,
			value: assets
		});
		return assets;
	}
});

Object.defineProperty(binding.Module.prototype, "identifier", {
	enumerable: true,
	configurable: true,
	value(this: binding.Module): string {
		return this[binding.MODULE_IDENTIFIER_SYMBOL];
	}
});
Object.defineProperty(binding.Module.prototype, "readableIdentifier", {
	enumerable: true,
	configurable: true,
	value(this: binding.Module) {
		return this._readableIdentifier;
	}
});
Object.defineProperty(binding.Module.prototype, "originalSource", {
	enumerable: true,
	configurable: true,
	value(this: binding.Module) {
		const originalSource = this._originalSource();
		if (originalSource) {
			return JsSource.__from_binding(originalSource);
		}
		return null;
	}
});
Object.defineProperty(binding.Module.prototype, "emitFile", {
	enumerable: true,
	configurable: true,
	value(
		this: binding.Module,
		filename: string,
		source: Source,
		assetInfo?: binding.AssetInfo
	) {
		return this._emitFile(filename, JsSource.__to_binding(source), assetInfo);
	}
});

declare module "@rspack/binding" {
	interface Module {
		identifier(): string;
		readableIdentifier(): string;
		originalSource(): Source | null;
		emitFile(filename: string, source: Source, assetInfo?: AssetInfo): void;
	}
}

export { Module } from "@rspack/binding";
