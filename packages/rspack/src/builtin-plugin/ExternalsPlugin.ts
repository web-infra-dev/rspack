import {
	type BuiltinPlugin,
	BuiltinPluginName,
	type RawExternalItemFnCtx,
	type RawExternalsPluginOptions
} from "@rspack/binding";

import type { Compiler, ExternalItem, ExternalItemValue, Externals } from "..";
import { Resolver } from "../Resolver";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";

export class ExternalsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.ExternalsPlugin;

	constructor(
		private type: string,
		private externals: Externals
	) {
		super();
	}

	raw(compiler: Compiler): BuiltinPlugin | undefined {
		const { type, externals } = this;
		const raw: RawExternalsPluginOptions = {
			type,
			externals: (Array.isArray(externals) ? externals : [externals])
				.filter(Boolean)
				.map(item => getRawExternalItem(compiler, item))
		};
		return createBuiltinPlugin(this.name, raw);
	}
}

type ArrayType<T> = T extends (infer R)[] ? R : never;
type RecordValue<T> = T extends Record<any, infer R> ? R : never;
type RawExternalItem = ArrayType<RawExternalsPluginOptions["externals"]>;
type RawExternalItemValue = RecordValue<RawExternalItem>;

function getRawExternalItem(
	compiler: Compiler,
	item: ExternalItem | undefined
): RawExternalItem {
	if (typeof item === "string" || item instanceof RegExp) {
		return item;
	}

	if (typeof item === "function") {
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
						getResolve: function getResolve(options) {
							const resolver = new Resolver(ctx.getResolver());
							const getResolveContext = () => ({
								fileDependencies: compiler._lastCompilation!.fileDependencies,
								missingDependencies:
									compiler._lastCompilation!.missingDependencies,
								contextDependencies:
									compiler._lastCompilation!.contextDependencies
							});
							const child = options ? resolver.withOptions(options) : resolver;
							return (context, request, callback) => {
								if (callback) {
									child.resolve(
										{},
										context,
										request,
										getResolveContext(),
										(err, result) => {
											if (err) return callback(err);
											// Sync with how webpack fixes the type:
											// https://github.com/webpack/webpack/blob/a2ad76cd50ae780dead395c68ea67d46de9828f3/lib/ExternalModuleFactoryPlugin.js#L276
											callback(
												undefined,
												typeof result === "string" ? result : undefined
											);
										}
									);
								} else {
									return new Promise((resolve, reject) => {
										child.resolve(
											{},
											context,
											request,
											getResolveContext(),
											(err, result) => {
												if (err) reject(err);
												else resolve(result);
											}
										);
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
}

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
