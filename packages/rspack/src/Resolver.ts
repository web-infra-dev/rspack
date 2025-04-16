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

export type ResourceData = binding.JsResourceData;

export type ResolveRequest = ResourceData;

export class Resolver {
	binding: binding.JsResolver;

	constructor(binding: binding.JsResolver) {
		this.binding = binding;
	}

	resolve(
		context: object,
		path: string,
		request: string,
		resolveContext: ResolveContext,
		callback: ResolveCallback
	): void {
		try {
			this.binding.resolve(path, request).then(
				data => {
					if (data === false) {
						callback(null, false);
						return;
					}
					callback(null, data.resource, data);
				},
				err => callback(err as ErrorWithDetails)
			);
		} catch (err) {
			callback(err as ErrorWithDetails);
		}
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
