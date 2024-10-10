import type { DependenciesBlockDTO } from "@rspack/binding";
import { Dependency } from "./Dependency";

export class DependenciesBlock {
	#binding: DependenciesBlockDTO;

	constructor(binding: DependenciesBlockDTO) {
		this.#binding = binding;
	}

	get dependencies(): Dependency[] {
		return this.#binding.dependencies.map(d => Dependency.__from_binding(d));
	}

	get blocks(): DependenciesBlock[] {
		return this.#binding.blocks.map(b => new DependenciesBlock(b));
	}
}
