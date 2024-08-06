import type { DependencyDTO } from "@rspack/binding";

export class Dependency {
	#binding: DependencyDTO;

	constructor(binding: DependencyDTO) {
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
}
