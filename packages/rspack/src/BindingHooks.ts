import { Buffer } from 'node:buffer';
import binding from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';
import type { Compiler } from './Compiler';

type FixedSizeArray<T extends number, U> = T extends 0
  ? undefined[]
  : ReadonlyArray<U> & {
      0: U;
      length: T;
    };
type AsArray<T> = T extends any[] ? T : [T];
type ArgumentNames<T extends any[]> = FixedSizeArray<T['length'], string>;
type DefaultAdditionalOptions =
  liteTapable.Hook<any, any> extends liteTapable.Hook<any, any, infer A>
    ? A
    : never;

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
}

export const COMPILER_HOOK_SUBSCRIPTION_BITSETS = new WeakMap<
  Compiler,
  HookSubscriptionBitset
>();

export const COMPILATION_HOOK_SUBSCRIPTION_BITSETS = new WeakMap<
  Compiler,
  HookSubscriptionBitset
>();

export const createCompilerHookSubscriptionBitset = () =>
  new HookSubscriptionBitset(
    binding.COMPILER_HOOK_SUBSCRIPTION_BITSET_BYTE_LENGTH,
  );

export const createCompilationHookSubscriptionBitset = () =>
  new HookSubscriptionBitset(
    binding.COMPILATION_HOOK_SUBSCRIPTION_BITSET_BYTE_LENGTH,
  );

export class BindingSyncHook<
  T = any,
  R = void,
  AdditionalOptions = DefaultAdditionalOptions,
> extends liteTapable.SyncHook<T, R, AdditionalOptions> {
  #subscriptionBitset: HookSubscriptionBitset;
  #bitIndex: number;

  constructor(
    args: ArgumentNames<AsArray<T>> | undefined,
    subscriptionBitset: HookSubscriptionBitset,
    bitIndex: number,
    name?: string,
  ) {
    super(args, name);
    this.#subscriptionBitset = subscriptionBitset;
    this.#bitIndex = bitIndex;
  }

  override tap(
    ...args: Parameters<liteTapable.SyncHook<T, R, AdditionalOptions>['tap']>
  ): ReturnType<liteTapable.SyncHook<T, R, AdditionalOptions>['tap']> {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tap(...args);
  }

  override intercept(
    ...args: Parameters<
      liteTapable.SyncHook<T, R, AdditionalOptions>['intercept']
    >
  ): ReturnType<liteTapable.SyncHook<T, R, AdditionalOptions>['intercept']> {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.intercept(...args);
  }
}

export class BindingSyncBailHook<
  T = any,
  R = any,
  AdditionalOptions = DefaultAdditionalOptions,
> extends liteTapable.SyncBailHook<T, R, AdditionalOptions> {
  #subscriptionBitset: HookSubscriptionBitset;
  #bitIndex: number;

  constructor(
    args: ArgumentNames<AsArray<T>> | undefined,
    subscriptionBitset: HookSubscriptionBitset,
    bitIndex: number,
    name?: string,
  ) {
    super(args, name);
    this.#subscriptionBitset = subscriptionBitset;
    this.#bitIndex = bitIndex;
  }

  override tap(
    ...args: Parameters<
      liteTapable.SyncBailHook<T, R, AdditionalOptions>['tap']
    >
  ): ReturnType<liteTapable.SyncBailHook<T, R, AdditionalOptions>['tap']> {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tap(...args);
  }

  override intercept(
    ...args: Parameters<
      liteTapable.SyncBailHook<T, R, AdditionalOptions>['intercept']
    >
  ): ReturnType<
    liteTapable.SyncBailHook<T, R, AdditionalOptions>['intercept']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.intercept(...args);
  }
}

export class BindingSyncWaterfallHook<
  T = any,
  AdditionalOptions = DefaultAdditionalOptions,
> extends liteTapable.SyncWaterfallHook<T, AdditionalOptions> {
  #subscriptionBitset: HookSubscriptionBitset;
  #bitIndex: number;

  constructor(
    args: ArgumentNames<AsArray<T>> | undefined,
    subscriptionBitset: HookSubscriptionBitset,
    bitIndex: number,
    name?: string,
  ) {
    super(args, name);
    this.#subscriptionBitset = subscriptionBitset;
    this.#bitIndex = bitIndex;
  }

  override tap(
    ...args: Parameters<
      liteTapable.SyncWaterfallHook<T, AdditionalOptions>['tap']
    >
  ): ReturnType<liteTapable.SyncWaterfallHook<T, AdditionalOptions>['tap']> {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tap(...args);
  }

  override intercept(
    ...args: Parameters<
      liteTapable.SyncWaterfallHook<T, AdditionalOptions>['intercept']
    >
  ): ReturnType<
    liteTapable.SyncWaterfallHook<T, AdditionalOptions>['intercept']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.intercept(...args);
  }
}

export class BindingAsyncParallelHook<
  T = any,
  AdditionalOptions = DefaultAdditionalOptions,
> extends liteTapable.AsyncParallelHook<T, AdditionalOptions> {
  #subscriptionBitset: HookSubscriptionBitset;
  #bitIndex: number;

  constructor(
    args: ArgumentNames<AsArray<T>> | undefined,
    subscriptionBitset: HookSubscriptionBitset,
    bitIndex: number,
    name?: string,
  ) {
    super(args, name);
    this.#subscriptionBitset = subscriptionBitset;
    this.#bitIndex = bitIndex;
  }

  override tap(
    ...args: Parameters<
      liteTapable.AsyncParallelHook<T, AdditionalOptions>['tap']
    >
  ): ReturnType<liteTapable.AsyncParallelHook<T, AdditionalOptions>['tap']> {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tap(...args);
  }

  override tapAsync(
    ...args: Parameters<
      liteTapable.AsyncParallelHook<T, AdditionalOptions>['tapAsync']
    >
  ): ReturnType<
    liteTapable.AsyncParallelHook<T, AdditionalOptions>['tapAsync']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tapAsync(...args);
  }

  override tapPromise(
    ...args: Parameters<
      liteTapable.AsyncParallelHook<T, AdditionalOptions>['tapPromise']
    >
  ): ReturnType<
    liteTapable.AsyncParallelHook<T, AdditionalOptions>['tapPromise']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tapPromise(...args);
  }

  override intercept(
    ...args: Parameters<
      liteTapable.AsyncParallelHook<T, AdditionalOptions>['intercept']
    >
  ): ReturnType<
    liteTapable.AsyncParallelHook<T, AdditionalOptions>['intercept']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.intercept(...args);
  }
}

export class BindingAsyncSeriesHook<
  T = any,
  AdditionalOptions = DefaultAdditionalOptions,
> extends liteTapable.AsyncSeriesHook<T, AdditionalOptions> {
  #subscriptionBitset: HookSubscriptionBitset;
  #bitIndex: number;

  constructor(
    args: ArgumentNames<AsArray<T>> | undefined,
    subscriptionBitset: HookSubscriptionBitset,
    bitIndex: number,
    name?: string,
  ) {
    super(args, name);
    this.#subscriptionBitset = subscriptionBitset;
    this.#bitIndex = bitIndex;
  }

  override tap(
    ...args: Parameters<
      liteTapable.AsyncSeriesHook<T, AdditionalOptions>['tap']
    >
  ): ReturnType<liteTapable.AsyncSeriesHook<T, AdditionalOptions>['tap']> {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tap(...args);
  }

  override tapAsync(
    ...args: Parameters<
      liteTapable.AsyncSeriesHook<T, AdditionalOptions>['tapAsync']
    >
  ): ReturnType<liteTapable.AsyncSeriesHook<T, AdditionalOptions>['tapAsync']> {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tapAsync(...args);
  }

  override tapPromise(
    ...args: Parameters<
      liteTapable.AsyncSeriesHook<T, AdditionalOptions>['tapPromise']
    >
  ): ReturnType<
    liteTapable.AsyncSeriesHook<T, AdditionalOptions>['tapPromise']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tapPromise(...args);
  }

  override intercept(
    ...args: Parameters<
      liteTapable.AsyncSeriesHook<T, AdditionalOptions>['intercept']
    >
  ): ReturnType<
    liteTapable.AsyncSeriesHook<T, AdditionalOptions>['intercept']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.intercept(...args);
  }
}

export class BindingAsyncSeriesBailHook<
  T = any,
  R = any,
  AdditionalOptions = DefaultAdditionalOptions,
> extends liteTapable.AsyncSeriesBailHook<T, R, AdditionalOptions> {
  #subscriptionBitset: HookSubscriptionBitset;
  #bitIndex: number;

  constructor(
    args: ArgumentNames<AsArray<T>> | undefined,
    subscriptionBitset: HookSubscriptionBitset,
    bitIndex: number,
    name?: string,
  ) {
    super(args, name);
    this.#subscriptionBitset = subscriptionBitset;
    this.#bitIndex = bitIndex;
  }

  override tap(
    ...args: Parameters<
      liteTapable.AsyncSeriesBailHook<T, R, AdditionalOptions>['tap']
    >
  ): ReturnType<
    liteTapable.AsyncSeriesBailHook<T, R, AdditionalOptions>['tap']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tap(...args);
  }

  override tapAsync(
    ...args: Parameters<
      liteTapable.AsyncSeriesBailHook<T, R, AdditionalOptions>['tapAsync']
    >
  ): ReturnType<
    liteTapable.AsyncSeriesBailHook<T, R, AdditionalOptions>['tapAsync']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tapAsync(...args);
  }

  override tapPromise(
    ...args: Parameters<
      liteTapable.AsyncSeriesBailHook<T, R, AdditionalOptions>['tapPromise']
    >
  ): ReturnType<
    liteTapable.AsyncSeriesBailHook<T, R, AdditionalOptions>['tapPromise']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tapPromise(...args);
  }

  override intercept(
    ...args: Parameters<
      liteTapable.AsyncSeriesBailHook<T, R, AdditionalOptions>['intercept']
    >
  ): ReturnType<
    liteTapable.AsyncSeriesBailHook<T, R, AdditionalOptions>['intercept']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.intercept(...args);
  }
}

export class BindingAsyncSeriesWaterfallHook<
  T = any,
  AdditionalOptions = DefaultAdditionalOptions,
> extends liteTapable.AsyncSeriesWaterfallHook<T, AdditionalOptions> {
  #subscriptionBitset: HookSubscriptionBitset;
  #bitIndex: number;

  constructor(
    args: ArgumentNames<AsArray<T>> | undefined,
    subscriptionBitset: HookSubscriptionBitset,
    bitIndex: number,
    name?: string,
  ) {
    super(args, name);
    this.#subscriptionBitset = subscriptionBitset;
    this.#bitIndex = bitIndex;
  }

  override tap(
    ...args: Parameters<
      liteTapable.AsyncSeriesWaterfallHook<T, AdditionalOptions>['tap']
    >
  ): ReturnType<
    liteTapable.AsyncSeriesWaterfallHook<T, AdditionalOptions>['tap']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tap(...args);
  }

  override tapAsync(
    ...args: Parameters<
      liteTapable.AsyncSeriesWaterfallHook<T, AdditionalOptions>['tapAsync']
    >
  ): ReturnType<
    liteTapable.AsyncSeriesWaterfallHook<T, AdditionalOptions>['tapAsync']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tapAsync(...args);
  }

  override tapPromise(
    ...args: Parameters<
      liteTapable.AsyncSeriesWaterfallHook<T, AdditionalOptions>['tapPromise']
    >
  ): ReturnType<
    liteTapable.AsyncSeriesWaterfallHook<T, AdditionalOptions>['tapPromise']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.tapPromise(...args);
  }

  override intercept(
    ...args: Parameters<
      liteTapable.AsyncSeriesWaterfallHook<T, AdditionalOptions>['intercept']
    >
  ): ReturnType<
    liteTapable.AsyncSeriesWaterfallHook<T, AdditionalOptions>['intercept']
  > {
    this.#subscriptionBitset.markSubscribed(this.#bitIndex);
    return super.intercept(...args);
  }
}
