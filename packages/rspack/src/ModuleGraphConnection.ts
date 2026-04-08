import binding, {
  ModuleGraphConnection as BindingModuleGraphConnection,
} from '@rspack/binding';

type ModuleGraphConnectionConstructor = typeof BindingModuleGraphConnection & {
  readonly TRANSITIVE_ONLY: typeof binding.TRANSITIVE_ONLY_SYMBOL;
  readonly CIRCULAR_CONNECTION: typeof binding.CIRCULAR_CONNECTION_SYMBOL;
};

export interface ModuleGraphConnection extends BindingModuleGraphConnection {}

export const ModuleGraphConnection =
  BindingModuleGraphConnection as ModuleGraphConnectionConstructor;

Object.defineProperties(ModuleGraphConnection, {
  TRANSITIVE_ONLY: {
    value: binding.TRANSITIVE_ONLY_SYMBOL,
    enumerable: true,
  },
  CIRCULAR_CONNECTION: {
    value: binding.CIRCULAR_CONNECTION_SYMBOL,
    enumerable: true,
  },
});

export type ConnectionState =
  | boolean
  | typeof ModuleGraphConnection.TRANSITIVE_ONLY
  | typeof ModuleGraphConnection.CIRCULAR_CONNECTION;
