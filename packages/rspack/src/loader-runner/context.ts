import type {
  JsLoaderContext,
  JsLoaderHookContext,
  JsLoaderOutput,
  RspackError,
} from '@rspack/binding';

/** One owned snapshot per entry, written back as the same object on return. */
export class LoaderContextState {
  readonly native: JsLoaderContext | JsLoaderHookContext;
  readonly state: JsLoaderContext['state'];
  #module?: JsLoaderContext['_module'];
  #loaderCache?: JsLoaderContext['__internal__loaderCache'];

  constructor(native: JsLoaderContext | JsLoaderHookContext) {
    this.native = native;
    this.state = native.state;
    for (const item of this.state.loaderItemStates) item.data ??= {};
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
  get loaderContextState() {
    return this.state.loaderContextState;
  }
  set loaderContextState(value: object | undefined) {
    this.state.loaderContextState = value;
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
    return '__internal__loaderCache' in this.native
      ? (this.#loaderCache ??= this.native.__internal__loaderCache)
      : undefined;
  }
  get resource() {
    return this.native.resource;
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
    return this.state.output
      ? this.state.output.content
      : 'content' in this.native
        ? this.native.content
        : null;
  }
  get sourceMap() {
    return this.state.output
      ? this.state.output.sourceMap
      : 'sourceMap' in this.native
        ? (this.native.sourceMap ?? undefined)
        : undefined;
  }
  get additionalData() {
    return this.state.output
      ? this.state.output.additionalData
      : 'additionalData' in this.native
        ? (this.native.additionalData ?? undefined)
        : undefined;
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
