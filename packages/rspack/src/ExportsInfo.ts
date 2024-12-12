import { JsExportsInfo } from "@rspack/binding";

export type RuntimeSpec = string | string[] | undefined;

export class ExportsInfo {
	#inner: JsExportsInfo;

	static __from_binding(binding: JsExportsInfo) {
		return new ExportsInfo(binding);
	}

	static __to_binding(module: ExportsInfo): JsExportsInfo {
		return module.#inner;
	}

	private constructor(binding: JsExportsInfo) {
		this.#inner = binding;
	}

	isUsed(runtime: RuntimeSpec): boolean {
		return this.#inner.isUsed(runtime);
	}

	isModuleUsed(runtime: RuntimeSpec): boolean {
		return this.#inner.isModuleUsed(runtime);
	}

	setUsedInUnknownWay(runtime: RuntimeSpec): boolean {
		return this.#inner.setUsedInUnknownWay(runtime);
	}
}
