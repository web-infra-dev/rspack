import { BuiltinPluginName, RegisterJsTapKind } from '@rspack/binding';
import type { JsRealContentHashPluginUpdateHashData } from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';

import { type Compilation, checkCompilation } from '../Compilation';
import type { CreatePartialRegisters } from '../taps/types';
import { create } from './base';

const RealContentHashPluginImpl = create(
  BuiltinPluginName.RealContentHashPlugin,
  () => {},
  'compilation',
);

// Observer taps may return void, but invoking the hook returns undefined
// when no tap supplies a hash. Keep these return types separate.
interface UpdateHashHookOptions extends liteTapable.Hook<
  [Buffer[], string],
  string | undefined
> {
  tap(
    options: liteTapable.Options,
    fn: (assets: Buffer[], oldHash: string) => string | void,
  ): void;
  withOptions(
    options: Parameters<liteTapable.Hook['withOptions']>[0],
  ): UpdateHashHookOptions;
}

interface UpdateHashHook extends liteTapable.SyncBailHook<
  [Buffer[], string],
  string | undefined
> {
  tap: UpdateHashHookOptions['tap'];
  withOptions: UpdateHashHookOptions['withOptions'];
}

export type RealContentHashPluginHooks = {
  updateHash: UpdateHashHook;
};

export const RealContentHashPlugin =
  RealContentHashPluginImpl as typeof RealContentHashPluginImpl & {
    getCompilationHooks: (
      compilation: Compilation,
    ) => RealContentHashPluginHooks;
  };

const compilationHooksMap: WeakMap<Compilation, RealContentHashPluginHooks> =
  new WeakMap();

RealContentHashPlugin.getCompilationHooks = (compilation: Compilation) => {
  checkCompilation(compilation);

  let hooks = compilationHooksMap.get(compilation);
  if (hooks === undefined) {
    hooks = {
      updateHash: new liteTapable.SyncBailHook([
        'assets',
        'oldHash',
      ]) as UpdateHashHook,
    };
    compilationHooksMap.set(compilation, hooks);
  }
  return hooks;
};

export const createRealContentHashPluginHooksRegisters: CreatePartialRegisters<
  'RealContentHashPlugin'
> = (getCompiler, createTap) => {
  return {
    registerRealContentHashPluginUpdateHashTaps: createTap(
      RegisterJsTapKind.RealContentHashPluginUpdateHash,
      function (): liteTapable.Hook<[Buffer[], string], string | undefined> {
        return RealContentHashPlugin.getCompilationHooks(
          getCompiler().__internal__get_compilation()!,
        ).updateHash;
      },
      function (queried) {
        return function ({
          assets,
          oldHash,
        }: JsRealContentHashPluginUpdateHashData) {
          return queried.call(assets, oldHash);
        };
      },
    ),
  };
};
