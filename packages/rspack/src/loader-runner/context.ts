import type { JsLoaderContext } from '@rspack/binding';

type LazyFields = Pick<
  JsLoaderContext,
  | 'dependencies'
  | 'addedDependencies'
  | 'removedDependencies'
  | 'loaderItems'
  | 'sourceMap'
  | 'additionalData'
  | '__internal__parseMeta'
>;

const wrappers = new WeakMap<JsLoaderContext, JsLoaderContextWrapper>();

/** Keeps lazy native snapshots local to one invocation of the JS runner. */
export class JsLoaderContextWrapper {
  readonly #binding: JsLoaderContext;
  #cache: { -readonly [K in keyof LazyFields]?: LazyFields[K] } = {};
  readonly resource: JsLoaderContext['resource'];
  readonly _module: JsLoaderContext['_module'];
  readonly hot: JsLoaderContext['hot'];

  private constructor(binding: JsLoaderContext) {
    this.#binding = binding;
    this.resource = binding.resource;
    this._module = binding._module;
    this.hot = binding.hot;
  }

  static from(binding: JsLoaderContext): JsLoaderContextWrapper {
    let wrapper = wrappers.get(binding);
    if (!wrapper) {
      wrapper = new JsLoaderContextWrapper(binding);
      wrappers.set(binding, wrapper);
    }
    return wrapper;
  }

  // These fields are own JS data properties on the native class. Forwarding
  // reads and writes does not invoke a native getter or setter.
  get content() {
    return this.#binding.content;
  }
  set content(value: JsLoaderContext['content']) {
    this.#binding.content = value;
  }
  get cacheable() {
    return this.#binding.cacheable;
  }
  set cacheable(value: boolean) {
    this.#binding.cacheable = value;
  }
  get loaderIndex() {
    return this.#binding.loaderIndex;
  }
  set loaderIndex(value: number) {
    this.#binding.loaderIndex = value;
  }
  get loaderState() {
    return this.#binding.loaderState;
  }
  get loaderChainStart() {
    return this.#binding.loaderChainStart;
  }
  get loaderChainEnd() {
    return this.#binding.loaderChainEnd;
  }
  get __internal__error() {
    return this.#binding.__internal__error;
  }
  set __internal__error(value: JsLoaderContext['__internal__error']) {
    this.#binding.__internal__error = value;
  }

  get dependencies() {
    return this.read('dependencies');
  }
  set dependencies(value: LazyFields['dependencies']) {
    this.#cache.dependencies = value;
  }
  get addedDependencies() {
    return this.read('addedDependencies');
  }
  set addedDependencies(value: LazyFields['addedDependencies']) {
    this.#cache.addedDependencies = value;
  }
  get removedDependencies() {
    return this.read('removedDependencies');
  }
  set removedDependencies(value: LazyFields['removedDependencies']) {
    this.#cache.removedDependencies = value;
  }
  get loaderItems() {
    return this.read('loaderItems');
  }
  set loaderItems(value: LazyFields['loaderItems']) {
    this.#cache.loaderItems = value;
  }
  get sourceMap() {
    return this.read('sourceMap');
  }
  set sourceMap(value: LazyFields['sourceMap']) {
    this.#cache.sourceMap = value;
  }
  get additionalData() {
    return this.read('additionalData');
  }
  set additionalData(value: LazyFields['additionalData']) {
    this.#cache.additionalData = value;
  }
  get __internal__parseMeta() {
    return this.read('__internal__parseMeta');
  }
  set __internal__parseMeta(value: LazyFields['__internal__parseMeta']) {
    this.#cache.__internal__parseMeta = value;
  }

  private read<K extends keyof LazyFields>(key: K): LazyFields[K] {
    // Membership, rather than a nullish check, also caches undefined values.
    if (!(key in this.#cache)) {
      this.#cache[key] = this.#binding[key];
    }
    return this.#cache[key] as LazyFields[K];
  }

  finish(): JsLoaderContext {
    try {
      // Materialize only accessed snapshots for Rust to merge. Untouched fields
      // remain native and retain their original values without conversion.
      for (const key of Object.keys(this.#cache) as (keyof LazyFields)[]) {
        Object.defineProperty(this.#binding, key, {
          value: this.#cache[key],
          writable: true,
          enumerable: true,
          configurable: true,
        });
      }
      return this.#binding;
    } finally {
      this.#cache = {};
    }
  }
}
