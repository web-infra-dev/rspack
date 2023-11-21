import { JsModule } from "@rspack/binding";

export class Module {
	#inner: JsModule;

	static __from_binding(module: JsModule) {
		return new Module(module);
	}

	constructor(module: JsModule) {
		this.#inner = module;
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
