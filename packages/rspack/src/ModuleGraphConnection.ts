import { JsModuleGraphConnection } from "@rspack/binding";
import { Module } from "./Module";
import { Dependency } from "./Dependency";

const MODULE_GRAPH_CONNECTION_MAPPINGS = new WeakMap<
	JsModuleGraphConnection,
	ModuleGraphConnection
>();

export class ModuleGraphConnection {
	declare readonly module: Module | null;
	declare readonly dependency: Dependency;

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

		Object.defineProperties(this, {
			module: {
				enumerable: true,
				get(): Module | null {
					return binding.module ? Module.__from_binding(binding.module) : null;
				}
			},
			dependency: {
				enumerable: true,
				get(): Dependency {
					return Dependency.__from_binding(binding.dependency);
				}
			}
		});
	}
}
