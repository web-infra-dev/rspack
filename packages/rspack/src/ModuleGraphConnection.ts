import {
  CIRCULAR_CONNECTION_SYMBOL,
  ModuleGraphConnection as BindingModuleGraphConnection,
  TRANSITIVE_ONLY_SYMBOL,
} from '@rspack/binding';

type ModuleGraphConnectionConstructor = typeof BindingModuleGraphConnection & {
  readonly TRANSITIVE_ONLY: typeof TRANSITIVE_ONLY_SYMBOL;
  readonly CIRCULAR_CONNECTION: typeof CIRCULAR_CONNECTION_SYMBOL;
};

export interface ModuleGraphConnection extends BindingModuleGraphConnection {}

export const ModuleGraphConnection =
  BindingModuleGraphConnection as ModuleGraphConnectionConstructor;

Object.defineProperties(ModuleGraphConnection, {
  TRANSITIVE_ONLY: {
    value: TRANSITIVE_ONLY_SYMBOL,
    enumerable: true,
  },
  CIRCULAR_CONNECTION: {
    value: CIRCULAR_CONNECTION_SYMBOL,
    enumerable: true,
  },
});

export type ConnectionState =
  | boolean
  | typeof ModuleGraphConnection.TRANSITIVE_ONLY
  | typeof ModuleGraphConnection.CIRCULAR_CONNECTION;
