import { JsDependency, JsCompiledDependency } from "@rspack/binding";

export class Dependency {
	#binding: JsDependency | JsCompiledDependency;

	static __from_binding(
		binding: JsDependency | JsCompiledDependency
	): Dependency {
		return new Dependency(binding);
	}

	private constructor(binding: JsDependency | JsCompiledDependency) {
		this.#binding = binding;
	}

	get type(): string {
		return this.#binding.type;
	}

	get category(): string {
		return this.#binding.category;
	}

	get request(): string | undefined {
		return this.#binding.request;
	}

	get critital(): boolean | undefined {
		return this.#binding.critical;
	}

	set critital(critital: boolean | undefined) {
		if (
			typeof critital === "boolean" &&
			this.#binding instanceof JsDependency
		) {
			this.#binding.critical = critital;
		}
	}
}
