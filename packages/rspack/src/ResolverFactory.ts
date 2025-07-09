import * as binding from "@rspack/binding";
import * as liteTapable from "@rspack/lite-tapable";
import { Resolver } from "./Resolver";
import { type Resolve, getRawResolve } from "./config";

type ResolveOptionsWithDependencyType = Resolve & {
	dependencyCategory?: string;
	resolveToContext?: boolean;
};

export class ResolverFactory {
	#binding: binding.JsResolverFactory;

	hooks: {
		resolveOptions: liteTapable.SyncWaterfallHook<
			[ResolveOptionsWithDependencyType, { type: string }]
		>;
		resolver: liteTapable.HookMap<
			liteTapable.SyncHook<
				[Resolver, ResolveOptionsWithDependencyType, { type: string }]
			>
		>;
	};

	static __to_binding(
		resolver_factory: ResolverFactory
	): binding.JsResolverFactory {
		return resolver_factory.#binding;
	}

	constructor(pnp: boolean) {
		this.#binding = new binding.JsResolverFactory(pnp);
		this.hooks = {
			resolveOptions: new liteTapable.SyncWaterfallHook([
				"resolveOptions",
				"context"
			]),
			resolver: new liteTapable.HookMap(
				() =>
					new liteTapable.SyncHook(["resolver", "resolveOptions", "context"])
			)
		};
	}

	get(
		type: string,
		resolveOptions?: ResolveOptionsWithDependencyType
	): Resolver {
		// Prepare context for hooks
		const context = { type };

		// Apply resolveOptions hook to allow modification of resolve options
		const resolveOptionsToUse = this.hooks.resolveOptions.call(
			resolveOptions || {},
			context
		);

		const { dependencyCategory, resolveToContext, ...resolve } =
			resolveOptionsToUse;

		const binding = this.#binding.get(type, {
			...getRawResolve(resolve),
			dependencyCategory,
			resolveToContext
		});

		const resolver = new Resolver(binding);

		// Call resolver hook to allow plugins to access the created resolver
		this.hooks.resolver.for(type).call(resolver, resolveOptionsToUse, context);

		return resolver;
	}
}
