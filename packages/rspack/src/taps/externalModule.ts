import binding from '@rspack/binding';
import { ExternalModule } from '../ExternalModule';
import type { CreatePartialRegisters } from './types';

export const createExternalModuleHooksRegisters: CreatePartialRegisters<
  'ExternalModule'
> = (getCompiler, createTap) => ({
  registerExternalModuleChunkConditionTaps: createTap(
    binding.RegisterJsTapKind.ExternalModuleChunkCondition,
    () =>
      ExternalModule.getCompilationHooks(
        getCompiler().__internal__get_compilation()!,
      ).chunkCondition,
    (queried) => (chunk: binding.Chunk) =>
      queried.call(chunk, getCompiler().__internal__get_compilation()!),
  ),
});
