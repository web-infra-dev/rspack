import type {
  JsLoaderContext,
  JsLoaderHookContext,
  JsLoaderOutput,
  RspackError,
} from '@rspack/binding';

const UNREAD = Symbol('unread');

/** Reused wrapper with one owned state snapshot and read cache per entry. */
export class LoaderContextState {
  native: JsLoaderContext | JsLoaderHookContext;
  state: JsLoaderContext['state'];
  #module?: JsLoaderContext['_module'];
  #resource?: string;
  // Reset on every native entry. Cache missing values too.
  #loaderCache: JsLoaderContext['__internal__loaderCache'] | typeof UNREAD =
    UNREAD;
  #content: JsLoaderContext['content'] | typeof UNREAD = UNREAD;
  #sourceMap: JsLoaderOutput['sourceMap'] | typeof UNREAD = UNREAD;
  #additionalData: JsLoaderOutput['additionalData'] | typeof UNREAD = UNREAD;

  constructor(native: JsLoaderContext | JsLoaderHookContext) {
    this.native = native;
    this.state = native.state;
    for (const item of this.state.loaderItemStates) item.data ??= {};
  }

  get identity(): JsLoaderContext {
    return 'identity' in this.native ? this.native.identity : this.native;
  }

  enter(native: JsLoaderContext | JsLoaderHookContext) {
    this.native = native;
    this.state = native.state;
    for (const item of this.state.loaderItemStates) item.data ??= {};
    this.#module = undefined;
    this.#resource = undefined;
    this.#loaderCache = UNREAD;
    this.#content = UNREAD;
    this.#sourceMap = UNREAD;
    this.#additionalData = UNREAD;
  }

  get loaderState() {
    return this.state.loaderState;
  }
  get loaderIndex() {
    return this.state.loaderIndex;
  }
  set loaderIndex(value: number) {
    this.state.loaderIndex = value;
  }
  get cacheable() {
    return this.state.cacheable;
  }
  set cacheable(value: boolean) {
    this.state.cacheable = value;
  }
  get dependencies() {
    return this.state.dependencies;
  }
  get __internal__parseMeta() {
    return this.state.parseMeta;
  }
  set __internal__error(value: RspackError) {
    this.state.error = value;
  }
  get __internal__loaderCache() {
    if (this.#loaderCache === UNREAD) {
      this.#loaderCache =
        '__internal__loaderCache' in this.native
          ? this.native.__internal__loaderCache
          : undefined;
    }
    return this.#loaderCache;
  }
  get resource() {
    return (this.#resource ??= this.native.resource);
  }
  get hot() {
    return this.state.hot;
  }
  set hot(value: boolean) {
    this.state.hot = value;
  }
  get _module() {
    return (this.#module ??= this.native._module);
  }
  get content() {
    if (this.state.output) return this.state.output.content;
    if (this.#content === UNREAD) {
      this.#content = 'content' in this.native ? this.native.content : null;
    }
    return this.#content;
  }
  get sourceMap() {
    if (this.state.output) return this.state.output.sourceMap;
    if (this.#sourceMap === UNREAD) {
      this.#sourceMap =
        'sourceMap' in this.native
          ? (this.native.sourceMap ?? undefined)
          : undefined;
    }
    return this.#sourceMap;
  }
  get additionalData() {
    if (this.state.output) return this.state.output.additionalData;
    if (this.#additionalData === UNREAD) {
      this.#additionalData =
        'additionalData' in this.native
          ? (this.native.additionalData ?? undefined)
          : undefined;
    }
    return this.#additionalData;
  }

  finish(output: JsLoaderOutput) {
    this.state.output = output;
  }

  commit() {
    this.native.state = this.state;
  }
}

/** Preserve ownership when a runner fails before it can commit its local state. */
export function setLoaderContextError(
  context: JsLoaderContext,
  error: unknown,
) {
  context.__internal__error = toLoaderContextError(error);
}

export function toLoaderContextError(error: unknown): RspackError {
  if (typeof error !== 'object' || error === null) {
    const wrapped = new Error(
      `(Emitted value instead of an instance of Error) ${String(error)}`,
    );
    wrapped.name = 'NonErrorEmittedError';
    return wrapped;
  } else {
    return error as RspackError;
  }
}

/** The native boundary always receives its class back, including on rejection. */
export async function runWithLoaderContext(
  context: JsLoaderContext,
  run: (context: JsLoaderContext) => unknown,
): Promise<JsLoaderContext> {
  try {
    await run(context);
  } catch (error) {
    setLoaderContextError(context, error);
  }
  return context;
}
