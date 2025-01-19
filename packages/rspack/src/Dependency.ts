import type { JsDependency } from "@rspack/binding";
import { VolatileValue } from "./util/volatile";

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
	#type: string | undefined;
	#category: string | undefined;
	#ids = new VolatileValue<string[] | null>();

	get type(): string {
		if (this.#type === undefined) {
			const binding = bindingDependencyFactory.getBinding(this);
			if (binding) {
				this.#type = binding.type;
			}
		}
		return this.#type || "unknown";
	}

	get category(): string {
		if (this.#category === undefined) {
			const binding = bindingDependencyFactory.getBinding(this);
			if (binding) {
				this.#category = binding.category;
			}
		}
		return this.#category || "unknown";
	}
	// biome-ignore lint/suspicious/useGetterReturn: shouldn't change binding signature
	get request(): string | undefined {
		const binding = bindingDependencyFactory.getBinding(this);
		if (binding) {
			return binding.request;
		}
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

	get ids(): string[] | null {
		const binding = bindingDependencyFactory.getBinding(this);
		if (this.#ids.has()) {
			return this.#ids.get()!;
		}
		if (binding) {
			const ids = binding.ids;
			this.#ids.set(ids);
			return ids;
		}
		return null;
	}
}
