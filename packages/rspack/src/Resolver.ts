import type binding from "@rspack/binding";
import type { ResolveCallback } from "./config/adapterRuleUse";

type ResolveContext = {};

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
}
