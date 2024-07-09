import type * as binding from "@rspack/binding";

interface ResolveContext {}

type ErrorWithDetail = Error & { details?: string };

type ResolveOptionsWithDependencyType = Omit<
	binding.RawResolveOptionsWithDependencyType,
	"restrictions"
> & {
	restrictions?: (string | RegExp)[];
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
		restrictions,
		...rest
	}: ResolveOptionsWithDependencyType): Resolver {
		const bindingOptions: binding.RawResolveOptionsWithDependencyType = rest;
		// TODO: rspack_resolver is unimplemented regex
		if (Array.isArray(restrictions)) {
			bindingOptions.restrictions = restrictions.filter<string>(isString);
		}
		const binding = this.binding.withOptions(bindingOptions);
		return new Resolver(binding);
	}
}
