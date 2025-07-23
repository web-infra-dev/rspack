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
	resolveRequestCache = new Map<string, ResolveRequest>();

	constructor(
		private type: string,
		private externals: Externals
	) {
		super();
	}

	raw(): BuiltinPlugin | undefined {
		const { type, externals } = this;
		const raw: RawExternalsPluginOptions = {
			type,
			externals: (Array.isArray(externals) ? externals : [externals])
				.filter(Boolean)
				.map(item => this.getRawExternalItem(item))
		};
		return createBuiltinPlugin(this.name, raw);
	}

	processResolveResult = (
		text: string | undefined
	): ResolveRequest | undefined => {
		if (!text) return undefined;

		let resolveRequest = this.resolveRequestCache.get(text);
		if (!resolveRequest) {
			resolveRequest = JSON.parse(text) as ResolveRequest;
			this.resolveRequestCache.set(text, resolveRequest);
		}
		return Object.assign({}, resolveRequest);
	};

	getRawExternalItem = (item: ExternalItem | undefined): RawExternalItem => {
		if (typeof item === "string" || item instanceof RegExp) {
			return item;
		}

		if (typeof item === "function") {
			const processResolveResult = this.processResolveResult;

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
							getResolve(options) {
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
												callback(null, req?.path ?? false, req);
											}
										});
									} else {
										return new Promise((promiseResolve, promiseReject) => {
											resolve(context, request, (error, text) => {
												if (error) {
													promiseReject(error);
												} else {
													const req = processResolveResult(text);
													promiseResolve(req?.path);
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
