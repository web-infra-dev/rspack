import { Buffer } from 'node:buffer';
import binding, { CompilerHooks } from '@rspack/binding';
import type { Compiler } from './Compiler';

export type HookSubscriptionKind =
  | binding.CompilerHooks
  | binding.CompilationHooks;

export class HookSubscriptionBitset {
  readonly buffer: Buffer;

  constructor(byteLength: number) {
    this.buffer = Buffer.alloc(byteLength);
  }

  markSubscribed(bitIndex: number) {
    this.buffer[bitIndex >> 3] |= 1 << (bitIndex & 7);
  }

  clear() {
    this.buffer.fill(0);
  }

  replaceSubscriptions(bitIndexes: Iterable<number>) {
    this.clear();
    for (const bitIndex of bitIndexes) {
      this.markSubscribed(bitIndex);
    }
  }
}

export const COMPILER_HOOK_SUBSCRIPTION_BITSETS = new WeakMap<
  Compiler,
  HookSubscriptionBitset
>();

export const COMPILATION_HOOK_SUBSCRIPTION_BITSETS = new WeakMap<
  Compiler,
  HookSubscriptionBitset
>();

const createHookSubscriptionBitset = (byteLength: number) =>
  new HookSubscriptionBitset(byteLength);

export const createCompilerHookSubscriptionBitset = () => {
  const bitset = createHookSubscriptionBitset(
    binding.COMPILER_HOOK_SUBSCRIPTION_BITSET_BYTE_LENGTH,
  );
  // ensure thisCompilation must call
  bitset.markSubscribed(CompilerHooks.ThisCompilation);
  return bitset;
};

export const createCompilationHookSubscriptionBitset = () =>
  createHookSubscriptionBitset(
    binding.COMPILATION_HOOK_SUBSCRIPTION_BITSET_BYTE_LENGTH,
  );
