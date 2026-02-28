import binding from '@rspack/binding';
import { getRawResolve, type Resolve } from './config';
import { Resolver } from './Resolver';
import { cachedCleverMerge } from './util/cleverMerge';

export type ResolveOptionsWithDependencyType = Resolve & {
  dependencyType?: string;
  resolveToContext?: boolean;
};

export type WithOptions = {
  withOptions: (
    options: ResolveOptionsWithDependencyType,
  ) => ResolverWithOptions;
};

export type ResolverWithOptions = Resolver & WithOptions;

type ResolverCache = {
  direct: WeakMap<ResolveOptionsWithDependencyType, ResolverWithOptions>;
  stringified: Map<string, ResolverWithOptions>;
};

const EMPTY_RESOLVE_OPTIONS: ResolveOptionsWithDependencyType = {};

export class ResolverFactory {
  #binding: binding.JsResolverFactory;
  #cache: Map<string, ResolverCache> = new Map();

  static __to_binding(
    resolver_factory: ResolverFactory,
  ): binding.JsResolverFactory {
    return resolver_factory.#binding;
  }

  constructor(
    pnp: boolean,
    resolveOptions: Resolve,
    loaderResolveOptions: Resolve,
  ) {
    this.#binding = new binding.JsResolverFactory(
      pnp,
      getRawResolve(resolveOptions),
      getRawResolve(loaderResolveOptions),
    );
  }

  #create(
    type: string,
    resolveOptionsWithDepType: ResolveOptionsWithDependencyType,
  ): ResolverWithOptions {
    const { dependencyType, resolveToContext, ...resolve } =
      resolveOptionsWithDepType;

    const binding = this.#binding.get(type, {
      ...getRawResolve(resolve),
      dependencyType,
      resolveToContext,
    });
    const resolver = new Resolver(binding) as ResolverWithOptions;
    const childCache = new WeakMap<
      ResolveOptionsWithDependencyType,
      ResolverWithOptions
    >();
    resolver.withOptions = (options: ResolveOptionsWithDependencyType) => {
      const cacheEntry = childCache.get(options);
      if (cacheEntry !== undefined) return cacheEntry;
      const mergedOptions = cachedCleverMerge(
        resolveOptionsWithDepType,
        options,
      );
      const newResolver = this.get(type, mergedOptions);
      childCache.set(options, newResolver);
      return newResolver;
    };
    return resolver;
  }

  get(
    type: string,
    resolveOptions: ResolveOptionsWithDependencyType = EMPTY_RESOLVE_OPTIONS,
  ): ResolverWithOptions {
    let typedCaches = this.#cache.get(type);
    if (!typedCaches) {
      typedCaches = {
        direct: new WeakMap(),
        stringified: new Map(),
      };
      this.#cache.set(type, typedCaches);
    }
    const cachedResolver = typedCaches.direct.get(resolveOptions);
    if (cachedResolver) {
      return cachedResolver;
    }
    const ident = JSON.stringify(resolveOptions);
    const resolver = typedCaches.stringified.get(ident);
    if (resolver) {
      typedCaches.direct.set(resolveOptions, resolver);
      return resolver;
    }
    const newResolver = this.#create(type, resolveOptions);
    typedCaches.direct.set(resolveOptions, newResolver);
    typedCaches.stringified.set(ident, newResolver);
    return newResolver;
  }
}
