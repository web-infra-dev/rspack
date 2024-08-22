import * as binding from "@rspack/binding";
import { Resolver } from "./Resolver";
import { type Resolve, getRawResolve } from "./config";

type ResolveOptionsWithDependencyType = Resolve & {
	dependencyCategory?: string;
	resolveToContext?: boolean;
};

export class ResolverFactory {
	#binding: binding.JsResolverFactory;
	// context to load pnp_manifest
	context: string;

	static __to_binding(
		resolver_factory: ResolverFactory
	): binding.JsResolverFactory {
		return resolver_factory.#binding;
	}

	constructor(context: string) {
		this.#binding = new binding.JsResolverFactory();
		this.context = context;
	}
	get(
		type: string,
		resolveOptions?: ResolveOptionsWithDependencyType
	): Resolver {
		const { dependencyCategory, resolveToContext, ...resolve } =
			resolveOptions || {};
		const binding = this.#binding.get(
			type,
			{
				...getRawResolve(resolve),
				dependencyCategory,
				resolveToContext
			},
			this.context
		);
		return new Resolver(binding);
	}
}
