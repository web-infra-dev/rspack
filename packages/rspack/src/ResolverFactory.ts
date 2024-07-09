import * as binding from "@rspack/binding";
import { Resolver } from "./Resolver";

type ResolveOptionsWithDependencyType =
	binding.RawResolveOptionsWithDependencyType;

export class ResolverFactory {
	#binding: binding.JsResolverFactory;

	static __to_binding(
		resolver_factory: ResolverFactory
	): binding.JsResolverFactory {
		return resolver_factory.#binding;
	}

	constructor() {
		this.#binding = new binding.JsResolverFactory();
	}

	get(
		type: string,
		resolveOptions?: ResolveOptionsWithDependencyType
	): Resolver {
		const binding = this.#binding.get(type, resolveOptions);
		return new Resolver(binding);
	}
}
