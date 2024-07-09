import type * as binding from "@rspack/binding";

interface ResolveContext {}

type ErrorWithDetail = Error & { details?: string };

type ResolveOptionsWithDependencyType =
	binding.RawResolveOptionsWithDependencyType;

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

	withOptions(options: ResolveOptionsWithDependencyType): Resolver {
		const binding = this.binding.withOptions(options);
		return new Resolver(binding);
	}
}
