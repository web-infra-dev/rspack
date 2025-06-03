import type * as binding from "@rspack/binding";
import { type Resolve, getRawResolve } from "./config";
import type { ResolveCallback } from "./config/adapterRuleUse";

type ResolveContext = {};

type ResolveOptionsWithDependencyType = Resolve & {
	dependencyCategory?: string;
	resolveToContext?: boolean;
};

export type ResourceData = binding.JsResourceData;

export type ResolveRequest = ResourceData;

export class Resolver {
	binding: binding.JsResolver;

	constructor(binding: binding.JsResolver) {
		this.binding = binding;
	}

	resolveSync(context: object, path: string, request: string): string | false {
		const data = this.binding.resolveSync(path, request);
		if (data === false) return data;
		return data.resource;
	}

	resolve(
		context: object,
		path: string,
		request: string,
		resolveContext: ResolveContext,
		callback: ResolveCallback
	): void {
		this.binding.resolve(path, request, (error, data) =>
			callback(error, data?.resource, data)
		);
	}

	withOptions({
		dependencyCategory,
		resolveToContext,
		...resolve
	}: ResolveOptionsWithDependencyType): Resolver {
		const rawResolve = getRawResolve(resolve);

		const binding = this.binding.withOptions({
			dependencyCategory,
			resolveToContext,
			...rawResolve
		});
		return new Resolver(binding);
	}
}
