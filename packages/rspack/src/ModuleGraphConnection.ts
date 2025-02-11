import type { JsModuleGraphConnection } from "@rspack/binding";
import { Dependency } from "./Dependency";
import { Module } from "./Module";

const MODULE_GRAPH_CONNECTION_MAPPINGS = new WeakMap<
	JsModuleGraphConnection,
	ModuleGraphConnection
>();

export class ModuleGraphConnection {
	readonly dependency: Dependency;
	readonly resolvedModule: Module | null;

	declare readonly module: Module | null;
	declare readonly originModule: Module | null;

	#inner: JsModuleGraphConnection;

	static __from_binding(binding: JsModuleGraphConnection) {
		let connection = MODULE_GRAPH_CONNECTION_MAPPINGS.get(binding);
		if (connection) {
			return connection;
		}
		connection = new ModuleGraphConnection(binding);
		MODULE_GRAPH_CONNECTION_MAPPINGS.set(binding, connection);
		return connection;
	}

	static __to_binding(data: ModuleGraphConnection): JsModuleGraphConnection {
		return data.#inner;
	}

	private constructor(binding: JsModuleGraphConnection) {
		this.#inner = binding;

		this.dependency = Dependency.__from_binding(binding.dependency);
		this.resolvedModule = binding.resolvedModule
			? Module.__from_binding(binding.resolvedModule)
			: null;

		Object.defineProperties(this, {
			module: {
				enumerable: true,
				get(): Module | null {
					return binding.module ? Module.__from_binding(binding.module) : null;
				}
			},
			originModule: {
				enumerable: true,
				get(): Module | null {
					return binding.originModule
						? Module.__from_binding(binding.originModule)
						: null;
				}
			}
		});
	}
}
