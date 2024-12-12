import { JsExportsInfo } from "@rspack/binding";

type RuntimeSpec = string | string[] | undefined;

type UsageStateType = 0 | 1 | 2 | 3 | 4;

export class ExportsInfo {
	#inner: JsExportsInfo;

	static __from_binding(binding: JsExportsInfo) {
		return new ExportsInfo(binding);
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

	getUsed(name: string | string[], runtime: RuntimeSpec): UsageStateType {
		return this.#inner.getUsed(name, runtime);
	}
}
