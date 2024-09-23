import { type JsDependency, JsDependencyMut } from "@rspack/binding";

export class Dependency {
	#binding: JsDependencyMut | JsDependency;
	#dropped = false;

	static __from_binding(binding: JsDependencyMut | JsDependency): Dependency {
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

	private constructor(binding: JsDependencyMut | JsDependency) {
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

	get critital(): boolean {
		this.ensureValidLifecycle();
		return this.#binding.critical;
	}

	set critital(critital: boolean) {
		this.ensureValidLifecycle();
		if (
			typeof critital === "boolean" &&
			this.#binding instanceof JsDependencyMut
		) {
			this.#binding.critical = critital;
		}
	}
}
