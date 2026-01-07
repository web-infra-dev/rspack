import { BuiltinPluginName } from '@rspack/binding';
import type { Module } from '../../Module';

import { create } from '../base';

export const BuiltinLazyCompilationPlugin = create(
  BuiltinPluginName.LazyCompilationPlugin,
  (
    currentActiveModules: () => Set<string>,
    entries: boolean,
    imports: boolean,
    client: string,
    test?: RegExp | ((module: Module) => boolean),
  ) => ({ module, imports, entries, test, client, currentActiveModules }),
  'thisCompilation',
);
