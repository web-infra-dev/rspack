import { JsExportsInfo } from "@rspack/binding";
import { RuntimeSpec } from "./util/runtime";

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

	setUsedInUnknownWay(runtime: RuntimeSpec): boolean {
		return this.#inner.setUsedInUnknownWay(runtime);
	}
}
