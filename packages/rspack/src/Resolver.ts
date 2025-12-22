import type binding from "@rspack/binding";
import type { ResolveCallback } from "./config/adapterRuleUse";

export type ResolveContext = {
	contextDependencies?: {
		add: (context: string) => void;
	};
	missingDependencies?: {
		add: (dependency: string) => void;
	};
	fileDependencies?: {
		add: (dependency: string) => void;
	};
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
	fileDependencies?: string[];
	missingDependencies?: string[];
	contextDependencies?: string[];
}

export class Resolver {
	#binding: binding.JsResolver;

	constructor(binding: binding.JsResolver) {
		this.#binding = binding;
	}

	resolveSync(_context: object, path: string, request: string): string | false {
		return this.#binding.resolveSync(path, request) ?? false;
	}

	resolve(
		_context: object,
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

			if (req?.fileDependencies) {
				req.fileDependencies.forEach(file => {
					resolveContext.fileDependencies?.add(file);
				});
			}

			if (req?.missingDependencies) {
				req.missingDependencies.forEach(missing => {
					resolveContext.missingDependencies?.add(missing);
				});
			}

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
