import binding from "@rspack/binding";
import { getRawResolve, type Resolve } from "./config";
import { Resolver } from "./Resolver";

type ResolveOptionsWithDependencyType = Resolve & {
	dependencyCategory?: string;
	resolveToContext?: boolean;
};

type ResolverCache = {
	direct: WeakMap<ResolveOptionsWithDependencyType, Resolver>;
	stringified: Map<string, Resolver>;
};

const EMPTY_RESOLVE_OPTIONS: ResolveOptionsWithDependencyType = {};

export class ResolverFactory {
	#binding: binding.JsResolverFactory;
	#cache: Map<string, ResolverCache> = new Map();

	static __to_binding(
		resolver_factory: ResolverFactory
	): binding.JsResolverFactory {
		return resolver_factory.#binding;
	}

	constructor(
		pnp: boolean,
		resolveOptons: Resolve,
		loaderResolveOptions: Resolve
	) {
		this.#binding = new binding.JsResolverFactory(
			pnp,
			getRawResolve(resolveOptons),
			getRawResolve(loaderResolveOptions)
		);
	}

	#create(
		type: string,
		resolveOptionsWithDepType: ResolveOptionsWithDependencyType
	): Resolver {
		const { dependencyCategory, resolveToContext, ...resolve } =
			resolveOptionsWithDepType;

		const binding = this.#binding.get(type, {
			...getRawResolve(resolve),
			dependencyCategory,
			resolveToContext
		});
		return new Resolver(binding);
	}

	get(
		type: string,
		resolveOptions: ResolveOptionsWithDependencyType = EMPTY_RESOLVE_OPTIONS
	): Resolver {
		let typedCaches = this.#cache.get(type);
		if (!typedCaches) {
			typedCaches = {
				direct: new WeakMap(),
				stringified: new Map()
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
