import type {
  JsLoaderContext,
  JsLoaderOutput,
  RspackError,
} from '@rspack/binding';

/** Local execution state; commit once each JavaScript hook or runner finishes. */
export class LoaderContextState {
  readonly native: JsLoaderContext;
  readonly loaderState: JsLoaderContext['loaderState'];
  loaderItems: JsLoaderContext['loaderItems'];
  loaderIndex: number;
  loaderContextState?: object;
  cacheable: boolean;
  dependencies: JsLoaderContext['dependencies'];
  __internal__parseMeta: Record<string, string> = {};
  __internal__error?: RspackError;
  readonly __internal__loaderCache: JsLoaderContext['__internal__loaderCache'];
  #module?: JsLoaderContext['_module'];
  #output?: JsLoaderOutput;

  constructor(native: JsLoaderContext) {
    this.native = native;
    this.loaderContextState = native.loaderContextState;
    this.loaderState = native.loaderState;
    this.loaderItems = native.loaderItems;
    this.loaderIndex = native.loaderIndex;
    this.cacheable = native.cacheable;
    this.dependencies = native.dependencies;
    this.__internal__loaderCache = native.__internal__loaderCache;
  }

  get resource() {
    return this.native.resource;
  }
  get hot() {
    return this.native.hot;
  }
  get _module() {
    return (this.#module ??= this.native._module);
  }
  get content() {
    return this.#output ? this.#output.content : this.native.content;
  }
  get sourceMap() {
    return this.#output
      ? this.#output.sourceMap
      : (this.native.sourceMap ?? undefined);
  }
  get additionalData() {
    return this.#output
      ? this.#output.additionalData
      : (this.native.additionalData ?? undefined);
  }

  finish(output: JsLoaderOutput) {
    this.#output = output;
  }

  commit() {
    this.native.__internal__result = {
      loaderContextState: this.loaderContextState,
      cacheable: this.cacheable,
      dependencies: this.dependencies,
      loaderItems: this.loaderItems,
      loaderIndex: this.loaderIndex,
      parseMeta: this.__internal__parseMeta,
      output: this.#output,
      error: this.__internal__error,
    };
  }
}
