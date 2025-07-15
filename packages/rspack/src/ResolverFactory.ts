import * as binding from "@rspack/binding";
import { getRawResolve, type Resolve } from "./config";
import { Resolver } from "./Resolver";

type ResolveOptionsWithDependencyType = Resolve & {
	dependencyCategory?: string;
	resolveToContext?: boolean;
};

export class ResolverFactory {
	#binding: binding.JsResolverFactory;

	static __to_binding(
		resolver_factory: ResolverFactory
	): binding.JsResolverFactory {
		return resolver_factory.#binding;
	}

	constructor(pnp: boolean) {
		this.#binding = new binding.JsResolverFactory(pnp);
	}

	get(
		type: string,
		resolveOptions?: ResolveOptionsWithDependencyType
	): Resolver {
		const { dependencyCategory, resolveToContext, ...resolve } =
			resolveOptions || {};

		const binding = this.#binding.get(type, {
			...getRawResolve(resolve),
			dependencyCategory,
			resolveToContext
		});
		return new Resolver(binding);
	}
}
