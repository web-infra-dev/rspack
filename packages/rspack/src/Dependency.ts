import type { JsDependency } from "@rspack/binding";

const TO_BINDING_MAPPINGS = new WeakMap<Dependency, JsDependency>();
const BINDING_MAPPINGS = new WeakMap<JsDependency, Dependency>();

// internal object
export const bindingDependencyFactory = {
	getBinding(dependency: Dependency): JsDependency | undefined {
		return TO_BINDING_MAPPINGS.get(dependency);
	},

	setBinding(dependency: Dependency, binding: JsDependency) {
		BINDING_MAPPINGS.set(binding, dependency);
		TO_BINDING_MAPPINGS.set(dependency, binding);
	},

	create(ctor: typeof Dependency, binding: JsDependency): Dependency {
		if (BINDING_MAPPINGS.has(binding)) {
			return BINDING_MAPPINGS.get(binding)!;
		}
		const dependency = Object.create(ctor);
		BINDING_MAPPINGS.set(binding, dependency);
		TO_BINDING_MAPPINGS.set(dependency, binding);
		return dependency;
	}
};

export class Dependency {
	declare readonly type: string;
	declare readonly category: string;
	declare readonly request: string | undefined;
	declare critical: boolean;

	constructor() {
		Object.defineProperties(this, {
			type: {
				enumerable: true,
				get(): string {
					const binding = bindingDependencyFactory.getBinding(this);
					if (binding) {
						return binding.type;
					}
					return "unknown";
				}
			},
			category: {
				enumerable: true,
				get(): string {
					const binding = bindingDependencyFactory.getBinding(this);
					if (binding) {
						return binding.category;
					}
					return "unknown";
				}
			},
			request: {
				enumerable: true,
				get(): string | undefined {
					const binding = bindingDependencyFactory.getBinding(this);
					if (binding) {
						return binding.request;
					}
				}
			},
			critical: {
				enumerable: true,
				get(): boolean {
					const binding = bindingDependencyFactory.getBinding(this);
					if (binding) {
						return binding.critical;
					}
					return false;
				},
				set(val: boolean) {
					const binding = bindingDependencyFactory.getBinding(this);
					if (binding) {
						binding.critical = val;
					}
				}
			}
		});
	}

	get ids(): string[] | undefined {
		const binding = bindingDependencyFactory.getBinding(this);
		if (binding) {
			return binding.ids;
		}
		return undefined;
	}
}
