import { BuiltinPluginName } from '@rspack/binding';
import type { Module } from '../../Module';

import { create } from '../base';

export const BuiltinLazyCompilationPlugin = create(
  BuiltinPluginName.LazyCompilationPlugin,
  (
    takeNewlyActiveModules: () => Set<string>,
    entries: boolean,
    imports: boolean,
    client: string,
    collectReservedExternals: (layer?: string | null) => string[],
    test?: RegExp | ((module: Module) => boolean),
  ) => ({
    imports,
    entries,
    test,
    client,
    takeNewlyActiveModules,
    collectReservedExternals,
  }),
  'thisCompilation',
);
