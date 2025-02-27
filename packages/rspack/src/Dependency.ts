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
		const dependency = new ctor();
		BINDING_MAPPINGS.set(binding, dependency);
		TO_BINDING_MAPPINGS.set(dependency, binding);
		return dependency;
	}
};

export class Dependency {
	get type(): string {
		const binding = bindingDependencyFactory.getBinding(this);
		if (binding) {
			return binding.type;
		}
		return "unknown";
	}

	get category(): string {
		const binding = bindingDependencyFactory.getBinding(this);
		if (binding) {
			return binding.category;
		}
		return "unknown";
	}

	get request(): string | undefined {
		const binding = bindingDependencyFactory.getBinding(this);
		if (binding) {
			return binding.request;
		}
		return undefined;
	}

	get critical(): boolean {
		const binding = bindingDependencyFactory.getBinding(this);
		if (binding) {
			return binding.critical;
		}
		return false;
	}

	set critical(val: boolean) {
		const binding = bindingDependencyFactory.getBinding(this);
		if (binding) {
			binding.critical = val;
		}
	}

	get ids(): string[] | undefined {
		const binding = bindingDependencyFactory.getBinding(this);
		if (binding) {
			return binding.ids;
		}
		return undefined;
	}
}
