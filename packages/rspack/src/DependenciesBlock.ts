import type { Dependency, JsDependenciesBlock } from "@rspack/binding";

export class DependenciesBlock {
	#binding: JsDependenciesBlock;

	declare readonly dependencies: Dependency[];
	declare readonly blocks: DependenciesBlock[];

	static __from_binding(binding: JsDependenciesBlock): DependenciesBlock {
		return new DependenciesBlock(binding);
	}

	static __to_binding(block: DependenciesBlock): JsDependenciesBlock {
		return block.#binding;
	}

	private constructor(binding: JsDependenciesBlock) {
		this.#binding = binding;

		Object.defineProperties(this, {
			dependencies: {
				enumerable: true,
				get(): Dependency[] {
					return binding.dependencies;
				}
			},
			blocks: {
				enumerable: true,
				get(): DependenciesBlock[] {
					return binding.blocks.map(b => DependenciesBlock.__from_binding(b));
				}
			}
		});
	}
}
