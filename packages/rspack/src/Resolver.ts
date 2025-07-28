import type binding from "@rspack/binding";
import { getRawResolve, type Resolve } from "./config";
import type { ResolveCallback } from "./config/adapterRuleUse";

type ResolveContext = {};

type ResolveOptionsWithDependencyType = Resolve & {
	dependencyCategory?: string;
	resolveToContext?: boolean;
};

export type ResourceData = binding.JsResourceData;

type JsonValueTypes =
	| null
	| string
	| number
	| boolean
	| JsonObjectTypes
	| JsonValueTypes[];

type JsonObjectTypes = { [index: string]: JsonValueTypes } & {
	[index: string]:
		| undefined
		| null
		| string
		| number
		| boolean
		| JsonObjectTypes
		| JsonValueTypes[];
};

export interface ResolveRequest {
	path: string;
	query: string;
	fragment: string;
	descriptionFileData?: string;
	descriptionFilePath?: string;
}

export class Resolver {
	#binding: binding.JsResolver;
	#childCache: WeakMap<Partial<ResolveOptionsWithDependencyType>, Resolver> =
		new WeakMap();

	constructor(binding: binding.JsResolver) {
		this.#binding = binding;
	}

	resolveSync(context: object, path: string, request: string): string | false {
		return this.#binding.resolveSync(path, request) ?? false;
	}

	resolve(
		context: object,
		path: string,
		request: string,
		resolveContext: ResolveContext,
		callback: ResolveCallback
	): void {
		this.#binding.resolve(path, request, (error, text) => {
			if (error) {
				callback(error);
				return;
			}
			const req = text ? (JSON.parse(text) as ResolveRequest) : undefined;
			callback(
				error,
				req
					? `${req.path.replace(/#/g, "\u200b#")}${req.query.replace(/#/g, "\u200b#")}${req.fragment}`
					: false,
				req
			);
		});
	}

	withOptions(options: ResolveOptionsWithDependencyType): Resolver {
		const cacheEntry = this.#childCache.get(options);
		if (cacheEntry !== undefined) {
			return cacheEntry;
		}

		const { dependencyCategory, resolveToContext, ...resolve } = options;
		const rawResolve = getRawResolve(resolve);

		const binding = this.#binding.withOptions({
			dependencyCategory,
			resolveToContext,
			...rawResolve
		});
		const resolver = new Resolver(binding);
		this.#childCache.set(options, resolver);
		return resolver;
	}
}
