import {
	CIRCULAR_CONNECTION_SYMBOL,
	TRANSITIVE_ONLY_SYMBOL,
	ModuleGraphConnection as BindingModuleGraphConnection,
} from '@rspack/binding';

export type ConnectionState =
	| boolean
	| typeof CIRCULAR_CONNECTION_SYMBOL
	| typeof TRANSITIVE_ONLY_SYMBOL;

const ModuleGraphConnection = BindingModuleGraphConnection as typeof BindingModuleGraphConnection & {
	TRANSITIVE_ONLY: typeof TRANSITIVE_ONLY_SYMBOL;
	CIRCULAR_CONNECTION: typeof CIRCULAR_CONNECTION_SYMBOL;
};

ModuleGraphConnection.TRANSITIVE_ONLY = TRANSITIVE_ONLY_SYMBOL;
ModuleGraphConnection.CIRCULAR_CONNECTION = CIRCULAR_CONNECTION_SYMBOL;

export { ModuleGraphConnection };
