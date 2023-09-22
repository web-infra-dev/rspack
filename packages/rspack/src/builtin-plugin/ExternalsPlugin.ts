import {
	RawExternalItem,
	RawExternalItemValue,
	RawExternalsPluginOptions
} from "@rspack/binding";
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

function getRawExternalItem(item: ExternalItem): RawExternalItem {
	if (typeof item === "string") {
		return { type: "string", stringPayload: item };
	}
	if (item instanceof RegExp) {
		return { type: "regexp", regexpPayload: item.source };
	}
	if (typeof item === "function") {
		return {
			type: "function",
			fnPayload: async ctx => {
				return await new Promise((resolve, reject) => {
					const promise = item(ctx, (err, result, type) => {
						if (err) reject(err);
						resolve({
							result: getRawExternalItemValueFormFnResult(result),
							external_type: type
						});
					}) as Promise<ExternalItemValue>;
					if (promise && promise.then) {
						promise.then(
							result =>
								resolve({
									result: getRawExternalItemValueFormFnResult(result),
									external_type: undefined
								}),
							e => reject(e)
						);
					}
				});
			}
		};
	}
	return {
		type: "object",
		objectPayload: Object.fromEntries(
			Object.entries(item).map(([k, v]) => [k, getRawExternalItemValue(v)])
		)
	};
}

function getRawExternalItemValueFormFnResult(result?: ExternalItemValue) {
	return result === undefined ? result : getRawExternalItemValue(result);
}

function getRawExternalItemValue(
	value: ExternalItemValue
): RawExternalItemValue {
	if (typeof value === "string") {
		return { type: "string", stringPayload: value };
	} else if (typeof value === "boolean") {
		return { type: "bool", boolPayload: value };
	} else if (Array.isArray(value)) {
		return {
			type: "array",
			arrayPayload: value
		};
	} else if (typeof value === "object" && value !== null) {
		return {
			type: "object",
			objectPayload: Object.fromEntries(
				Object.entries(value).map(([k, v]) => [k, Array.isArray(v) ? v : [v]])
			)
		};
	}
	throw new Error("unreachable");
}
