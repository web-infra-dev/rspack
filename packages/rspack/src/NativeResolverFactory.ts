import { ResolveContext, ResolveRequest, Resolver } from "enhanced-resolve";
import { type Compilation } from ".";
import { getRawResolve } from "./config/adapter";
import ResolverFactory from "./ResolverFactory";
import { reserveNativeResolverOptions } from "./ResolverFactory";
import {
	RawResolveOptionsWithDependencyType,
	type JsResolver
} from "@rspack/binding";
import assert from "assert";
import { isNil } from "./util";

export class NativeResolverFactory extends ResolverFactory {
	compilation: Compilation | undefined;

	constructor() {
		super();
	}

	get(type: string, resolveOptions?: any): ResolverFactory.ResolverWithOptions {
		const { compilation } = this;
		assert(!isNil(compilation), "make sure compilation had been binding");
		return super.get(
			type,
			reserveNativeResolverOptions(resolveOptions),
			options => {
				return nativeResolverFactory(compilation, options);
			}
		);
	}
}

const ERROR_PREFIX = "error:";

class NativeResolver implements Resolver {
	resolver: JsResolver;
	constructor(resolver: JsResolver) {
		this.resolver = resolver;
	}
	get fileSystem(): any {
		throw Error("The filed of `fileSystem` is not available in nativeResolver");
	}
	get options(): any {
		throw Error("The filed of `options` is not available in nativeResolver");
	}
	get hooks(): any {
		throw Error("The filed of `hooks` is not available in nativeResolver");
	}
	ensureHook(): any {
		throw Error(
			"The method of `ensureHook` is not available in nativeResolver"
		);
	}
	getHook(): any {
		throw Error("The method of `getHook` is not available in nativeResolver");
	}
	resolveSync(_context: object, path: string, request: string): string | false {
		const res = this.resolver.resolve(path, request);
		if (typeof res === "boolean") {
			assert(res === false);
			return res;
		} else if (res.startsWith(ERROR_PREFIX)) {
			throw Error(res.slice(ERROR_PREFIX.length));
		} else {
			return res;
		}
	}
	resolve(
		_context: object,
		path: string,
		request: string,
		_resolveContext: ResolveContext,
		callback: (
			arg0: Error | null,
			arg1?: string | false | undefined,
			arg2?: ResolveRequest | undefined
		) => void
	): void {
		const res = this.resolver.resolve(path, request);
		if (typeof res === "boolean") {
			assert(res === false);
			callback(null, false);
		} else if (res.startsWith(ERROR_PREFIX)) {
			const error = Error(res.slice(ERROR_PREFIX.length));
			callback(error);
		} else {
			callback(null, res);
		}
	}
	doResolve() {
		throw Error("The method of `doResolve` is not available in nativeResolver");
	}
	parse(): any {
		throw Error("The method of `parse` is not available in nativeResolver");
	}
	isModule(): any {
		throw Error("The method of `isModule` is not available in nativeResolver");
	}
	isPrivate(): any {
		throw Error("The method of `isPrivate` is not available in nativeResolver");
	}
	isDirectory(): any {
		throw Error(
			"The method of `isDIrectoy` is not available in nativeResolver"
		);
	}
	join(): any {
		throw Error("The method of `join` is not available in nativeResolver");
	}
	normalize(): any {
		throw Error("The method of `normalize` is not available in nativeResolver");
	}
}

function nativeResolverFactory(
	compilation: Compilation,
	options: any
): Resolver {
	const raw: RawResolveOptionsWithDependencyType = {
		resolve: getRawResolve(options),
		dependencyCategory: options.dependencyType,
		resolveToContext: options.resolveToContext
	};
	const nativeResolver = compilation.__internal_create_native_resolver(raw);
	return new NativeResolver(nativeResolver);
}
