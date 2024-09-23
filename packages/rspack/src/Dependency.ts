import { JsDependency, JsCompiledDependency } from "@rspack/binding";

export class Dependency {
	#binding: JsDependency | JsCompiledDependency;
	#dropped = false;

	static __from_binding(
		binding: JsDependency | JsCompiledDependency
	): Dependency {
		return new Dependency(binding);
	}

	static __drop(dependency: Dependency) {
		dependency.#dropped = true;
	}

	private ensureValidLifecycle() {
		if (this.#dropped) {
			throw new Error(
				"The Dependency has exceeded its lifecycle and has been dropped by Rust."
			);
		}
	}

	private constructor(binding: JsDependency | JsCompiledDependency) {
		this.#binding = binding;
	}

	get type(): string {
		this.ensureValidLifecycle();
		return this.#binding.type;
	}

	get category(): string {
		this.ensureValidLifecycle();
		return this.#binding.category;
	}

	get request(): string | undefined {
		this.ensureValidLifecycle();
		return this.#binding.request;
	}

	get critital(): boolean | undefined {
		this.ensureValidLifecycle();
		return this.#binding.critical;
	}

	set critital(critital: boolean | undefined) {
		this.ensureValidLifecycle();
		if (
			typeof critital === "boolean" &&
			this.#binding instanceof JsDependency
		) {
			this.#binding.critical = critital;
		}
	}
}
