import {
	type BuiltinPlugin,
	BuiltinPluginName,
	type RawExternalItemFnCtx,
	type RawExternalsPluginOptions
} from "@rspack/binding";

import type { ExternalItem, ExternalItemValue, Externals } from "..";
import { getRawResolve } from "../config/adapter";
import type { ResolveCallback } from "../config/adapterRuleUse";
import type { ResolveRequest } from "../Resolver";
import { createBuiltinPlugin, RspackBuiltinPlugin } from "./base";

export class ExternalsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.ExternalsPlugin;

	#resolveRequestCache = new Map<string, ResolveRequest>();

	constructor(
		private type: string,
		private externals: Externals,
		private placeInInitial?: boolean
	) {
		super();
	}

	raw(): BuiltinPlugin | undefined {
		const type = this.type;
		const externals = this.externals;
		const raw: RawExternalsPluginOptions = {
			type,
			externals: (Array.isArray(externals) ? externals : [externals])
				.filter(Boolean)
				.map(item => this.#getRawExternalItem(item)),
			placeInInitial: this.placeInInitial ?? false
		};
		return createBuiltinPlugin(this.name, raw);
	}

	#processResolveResult = (
		text: string | undefined
	): ResolveRequest | undefined => {
		if (!text) return undefined;

		let resolveRequest = this.#resolveRequestCache.get(text);
		if (!resolveRequest) {
			resolveRequest = JSON.parse(text) as ResolveRequest;
			this.#resolveRequestCache.set(text, resolveRequest);
		}
		return Object.assign({}, resolveRequest);
	};

	// Reference: webpack/enhanced-resolve#255
	// Handle fragment escaping in resolve results:
	// - `#` can be escaped as `\0#` to prevent fragment parsing
	// - enhanced-resolve resolves `#` ambiguously as both path and fragment
	// - Example: `./some#thing` could resolve to `.../some.js#thing` or `.../some#thing.js`
	// - When `#` is part of the path, it gets escaped as `\0#` in the result
	// - We replace `\0#` with zero-width space + `#` (\u200b#) for compatibility
	#processRequest(req: ResolveRequest): string {
		return `${req.path.replace(/#/g, "\u200b#")}${req.query.replace(/#/g, "\u200b#")}${req.fragment}`;
	}

	#getRawExternalItem = (item: ExternalItem | undefined): RawExternalItem => {
		if (typeof item === "string" || item instanceof RegExp) {
			return item;
		}

		if (typeof item === "function") {
			const processResolveResult = this.#processResolveResult;

			return async (ctx: RawExternalItemFnCtx) => {
				return await new Promise((resolve, reject) => {
					const data = ctx.data();
					const promise = item(
						{
							request: data.request,
							dependencyType: data.dependencyType,
							context: data.context,
							contextInfo: {
								issuer: data.contextInfo.issuer,
								issuerLayer: data.contextInfo.issuerLayer ?? null
							},
							getResolve: options => {
								const rawResolve = options ? getRawResolve(options) : undefined;
								const resolve = ctx.getResolve(rawResolve);

								return (
									context: string,
									request: string,
									callback?: ResolveCallback
								) => {
									if (callback) {
										resolve(context, request, (error, text) => {
											if (error) {
												callback(error);
											} else {
												const req = processResolveResult(text);
												callback(
													null,
													req ? this.#processRequest(req) : false,
													req
												);
											}
										});
									} else {
										return new Promise((promiseResolve, promiseReject) => {
											resolve(context, request, (error, text) => {
												if (error) {
													promiseReject(error);
												} else {
													const req = processResolveResult(text);
													promiseResolve(
														req ? this.#processRequest(req) : undefined
													);
												}
											});
										});
									}
								};
							}
						},
						(err, result, type) => {
							if (err) reject(err);
							resolve({
								result: getRawExternalItemValueFormFnResult(result),
								externalType: type
							});
						}
					) as Promise<ExternalItemValue> | ExternalItemValue | undefined;
					if ((promise as Promise<ExternalItemValue>)?.then) {
						(promise as Promise<ExternalItemValue>).then(
							result =>
								resolve({
									result: getRawExternalItemValueFormFnResult(result),
									externalType: undefined
								}),
							e => reject(e)
						);
					} else if (item.length === 1) {
						// No callback and no promise returned, regarded as a synchronous function
						resolve({
							result: getRawExternalItemValueFormFnResult(
								promise as ExternalItemValue | undefined
							),
							externalType: undefined
						});
					}
				});
			};
		}
		if (typeof item === "object") {
			return Object.fromEntries(
				Object.entries(item).map(([k, v]) => [k, getRawExternalItemValue(v)])
			);
		}
		throw new TypeError(`Unexpected type of external item: ${typeof item}`);
	};
}

type ArrayType<T> = T extends (infer R)[] ? R : never;
type RecordValue<T> = T extends Record<any, infer R> ? R : never;
type RawExternalItem = ArrayType<RawExternalsPluginOptions["externals"]>;
type RawExternalItemValue = RecordValue<RawExternalItem>;

function getRawExternalItemValueFormFnResult(result?: ExternalItemValue) {
	return result === undefined ? result : getRawExternalItemValue(result);
}

function getRawExternalItemValue(
	value: ExternalItemValue
): RawExternalItemValue {
	if (value && typeof value === "object" && !Array.isArray(value)) {
		return Object.fromEntries(
			Object.entries(value).map(([k, v]) => [k, Array.isArray(v) ? v : [v]])
		);
	}
	return value;
}
