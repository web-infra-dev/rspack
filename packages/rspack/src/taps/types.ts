import type binding from '@rspack/binding';
import type * as liteTapable from '@rspack/lite-tapable';
import type { Compiler } from '../Compiler';

type CreateHookMapRegisterTaps = <H extends liteTapable.Hook<any, any, any>>(
  registerKind: binding.RegisterJsTapKind,
  getHookMap: () => liteTapable.HookMap<H>,
  createTap: (queried: liteTapable.QueriedHookMap<H>) => any,
) => (stages: number[]) => binding.JsTap[];

type CreateHookRegisterTaps = <T, R, A>(
  registerKind: binding.RegisterJsTapKind,
  getHook: () => liteTapable.Hook<T, R, A>,
  createTap: (queried: liteTapable.QueriedHook<T, R, A>) => any,
) => (stages: number[]) => binding.JsTap[];

// type CompilationRegisterJsTapKeys = `registerCompilation${string}Taps`;
type RegisterTapKeys<
  T,
  L extends string,
> = T extends keyof binding.RegisterJsTaps ? (T extends L ? T : never) : never;
type PartialRegisters<L extends string> = {
  [K in RegisterTapKeys<
    keyof binding.RegisterJsTaps,
    `register${L}${string}Taps`
  >]: binding.RegisterJsTaps[K];
};

export type CreatePartialRegisters<L extends string> = (
  getCompiler: () => Compiler,
  createTap: CreateHookRegisterTaps,
  createMapTap: CreateHookMapRegisterTaps,
) => PartialRegisters<L>;
