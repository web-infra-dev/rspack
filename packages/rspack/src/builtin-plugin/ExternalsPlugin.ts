import { RawExternalsPluginOptions } from "@rspack/binding";
import { BuiltinPluginName, create } from "./base";
import { ExternalItem, ExternalItemValue, Externals } from "..";

export const ExternalsPlugin = create(
	BuiltinPluginName.ExternalsPlugin,
	(type: string, externals: Externals): RawExternalsPluginOptions => {
		return {
			type,
			externals: (Array.isArray(externals) ? externals : [externals]).map(
				getRawExternalItem
			)
		};
	}
);

type ArrayType<T> = T extends (infer R)[] ? R : never;
type RecordValue<T> = T extends Record<any, infer R> ? R : never;
type RawExternalItem = ArrayType<RawExternalsPluginOptions["externals"]>;
type RawExternalItemValue = RecordValue<RawExternalItem>;

function getRawExternalItem(item: ExternalItem): RawExternalItem {
	if (typeof item === "string" || item instanceof RegExp) {
		return item;
	}

	if (typeof item === "function") {
		return async ctx => {
			return await new Promise((resolve, reject) => {
				const promise = item(ctx, (err, result, type) => {
					if (err) reject(err);
					resolve({
						result: getRawExternalItemValueFormFnResult(result),
						externalType: type
					});
				}) as Promise<ExternalItemValue>;
				if (promise && promise.then) {
					promise.then(
						result =>
							resolve({
								result: getRawExternalItemValueFormFnResult(result),
								externalType: undefined
							}),
						e => reject(e)
					);
				}
			});
		};
	}
	return Object.fromEntries(
		Object.entries(item).map(([k, v]) => [k, getRawExternalItemValue(v)])
	);
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
