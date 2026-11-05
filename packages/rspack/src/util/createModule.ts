import { ConcatenatedModule } from '../ConcatenatedModule';
import { ContextModule } from '../ContextModule';
import { ExternalModule } from '../ExternalModule';
import { Module } from '../Module';
import { NormalModule } from '../NormalModule';

const MODULE_CLASSES = [
  Module,
  NormalModule,
  ConcatenatedModule,
  ContextModule,
  ExternalModule,
];

// Passed through Compiler.jsHelpers. Rust pins the owning Compiler before this
// call, then wraps the returned instance. This function captures no Compiler.
export function createModule(kind: number): Module {
  return new MODULE_CLASSES[kind]();
}
