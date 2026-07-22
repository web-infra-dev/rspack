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

export type RealContentHashPluginHooks = {
  updateHash: liteTapable.SyncBailHook<[Buffer[], string], string | undefined>;
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
      updateHash: new liteTapable.SyncBailHook(['assets', 'oldHash']),
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
      function () {
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
