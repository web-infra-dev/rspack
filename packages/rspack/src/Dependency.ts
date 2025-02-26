import { EntryDependency, JsDependency } from "@rspack/binding";

export class Dependency {
	#inner: JsDependency;

	static [Symbol.hasInstance](instance: any) {
		return (
			instance instanceof EntryDependency || instance instanceof JsDependency
		);
	}

	declare readonly type: string;
	declare readonly category: string;
	declare readonly request: string | undefined;
	declare critical: boolean;

	static __from_binding(binding: JsDependency): Dependency {
		return new Dependency(binding);
	}

	static __to_binding(data: Dependency): JsDependency {
		if (data instanceof EntryDependency) {
			return data;
		}
		return data.#inner;
	}

	private constructor(binding: JsDependency) {
		this.#inner = binding;

		Object.defineProperties(this, {
			type: {
				enumerable: true,
				get(): string {
					return binding.type;
				}
			},
			category: {
				enumerable: true,
				get(): string {
					return binding.category;
				}
			},
			request: {
				enumerable: true,
				get(): string | undefined {
					return binding.request;
				}
			},
			critical: {
				enumerable: true,
				get(): boolean {
					return binding.critical;
				},
				set(val: boolean) {
					binding.critical = val;
				}
			}
		});
	}

	get ids(): string[] | undefined {
		return this.#inner.ids;
	}
}
