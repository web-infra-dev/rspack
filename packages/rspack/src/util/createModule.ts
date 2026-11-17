import { ModuleKind } from '@rspack/binding';
import { ConcatenatedModule } from '../ConcatenatedModule';
import { ContextModule } from '../ContextModule';
import { ExternalModule } from '../ExternalModule';
import { Module } from '../Module';
import { NormalModule } from '../NormalModule';
import { markAllocativeObject } from './allocative';

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
  const module = new MODULE_CLASSES[kind]();
  if (SUPPORTS_ALLOCATIVE) {
    markAllocativeObject(module, 'Module');
  }
  return module;
}
