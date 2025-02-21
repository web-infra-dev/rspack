import type * as binding from "@rspack/binding";
import { type Resolve, getRawResolve } from "./config";
import type {
	ErrorWithDetails,
	ResolveCallback
} from "./config/adapterRuleUse";

type ResolveContext = {};

type ResolveOptionsWithDependencyType = Resolve & {
	dependencyCategory?: string;
	resolveToContext?: boolean;
};

function isString(value: string | RegExp): value is string {
	return typeof value === "string";
}

const RESOLVER_MAPPINGS = new WeakMap<binding.JsResolver, Resolver>();

export class Resolver {
	#binding: binding.JsResolver;
	#cache = new WeakMap();

	static __from_binding(binding: binding.JsResolver) {
		let chunk = RESOLVER_MAPPINGS.get(binding);
		if (chunk) {
			return chunk;
		}
		chunk = new Resolver(binding);
		RESOLVER_MAPPINGS.set(binding, chunk);
		return chunk;
	}

	static __to_binding(chunk: Resolver): binding.JsResolver {
		return chunk.#binding;
	}

	private constructor(binding: binding.JsResolver) {
		this.#binding = binding;
	}

	resolveSync(context: object, path: string, request: string): string | false {
		return this.#binding.resolveSync(path, request);
	}

	resolve(
		context: object,
		path: string,
		request: string,
		resolveContext: ResolveContext,
		callback: ResolveCallback
	): void {
		try {
			const res = this.#binding.resolveSync(path, request);
			callback(null, res);
		} catch (err) {
			callback(err as ErrorWithDetails);
		}
	}

	withOptions(options: ResolveOptionsWithDependencyType): Resolver {
		const cacheEntry = this.#cache.get(options);
		if (cacheEntry !== undefined) return cacheEntry;

		const { dependencyCategory, resolveToContext, ...resolve } = options;
		const rawResolve = getRawResolve(resolve);

		// TODO: rspack_resolver is unimplemented regex
		if (Array.isArray(rawResolve.restrictions)) {
			rawResolve.restrictions =
				rawResolve.restrictions.filter<string>(isString);
		}

		const binding = this.#binding.withOptions({
			dependencyCategory,
			resolveToContext,
			...rawResolve
		});
		const resolver = new Resolver(binding);
		this.#cache.set(options, resolver);
		return new Resolver(binding);
	}
}
