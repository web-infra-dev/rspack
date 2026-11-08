import { ModuleKind } from '@rspack/binding';
import { ConcatenatedModule } from '../ConcatenatedModule';
import { ContextModule } from '../ContextModule';
import { ExternalModule } from '../ExternalModule';
import { Module } from '../Module';
import { NormalModule } from '../NormalModule';

const MODULE_CLASSES = {
  [ModuleKind.Module]: Module,
  [ModuleKind.Normal]: NormalModule,
  [ModuleKind.Concatenated]: ConcatenatedModule,
  [ModuleKind.Context]: ContextModule,
  [ModuleKind.External]: ExternalModule,
} satisfies Record<ModuleKind, new () => Module>;

// Passed through Compiler.jsHelpers. Rust pins the owning Compiler before this
// call, then wraps the returned instance. This function captures no Compiler.
export function createModule(kind: ModuleKind): Module {
  return new MODULE_CLASSES[kind]();
}
