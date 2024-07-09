import type * as binding from "@rspack/binding";
import { Resolve, getRawResolve } from "./config";

interface ResolveContext {}

type ErrorWithDetail = Error & { details?: string };

type ResolveOptionsWithDependencyType = Resolve & {
	dependencyCategory?: string;
	resolveToContext?: boolean;
};

function isString(value: string | RegExp): value is string {
	return typeof value === "string";
}

export class Resolver {
	binding: binding.JsResolver;

	constructor(binding: binding.JsResolver) {
		this.binding = binding;
	}

	resolveSync(context: object, path: string, request: string): string | false {
		return this.binding.resolveSync(path, request);
	}

	resolve(
		context: object,
		path: string,
		request: string,
		resolveContext: ResolveContext,
		callback: (
			err: null | ErrorWithDetail,
			res?: string | false
			// req?: ResolveRequest
		) => void
	): void {
		try {
			const res = this.binding.resolveSync(path, request);
			callback(null, res);
		} catch (err) {
			callback(err as ErrorWithDetail);
		}
	}

	withOptions({
		dependencyCategory,
		resolveToContext,
		...resolve
	}: ResolveOptionsWithDependencyType): Resolver {
		const rawResolve = getRawResolve(resolve);

		// TODO: rspack_resolver is unimplemented regex
		if (Array.isArray(rawResolve.restrictions)) {
			rawResolve.restrictions =
				rawResolve.restrictions.filter<string>(isString);
		}

		const binding = this.binding.withOptions({
			dependencyCategory,
			resolveToContext,
			...rawResolve
		});
		return new Resolver(binding);
	}
}
