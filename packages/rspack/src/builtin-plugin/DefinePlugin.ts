import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export type DefinePluginOptions = Record<string, CodeValue>;
export const DefinePlugin = create(
	BuiltinPluginName.DefinePlugin,
	function (define: DefinePluginOptions): NormalizedCodeValue {
		const supportsBigIntLiteral =
			this.options.output.environment?.bigIntLiteral ?? false;
		return normalizeValue(define, supportsBigIntLiteral);
	},
	"compilation"
);

const normalizeValue = (
	define: DefinePluginOptions,
	supportsBigIntLiteral: boolean
) => {
	const normalizePrimitive = (
		p: CodeValuePrimitive
	): NormalizedCodeValuePrimitive => {
		if (p === undefined) {
			return "undefined";
		} else if (Object.is(p, -0)) {
			return "-0";
		} else if (p instanceof RegExp) {
			return p.toString();
		} else if (typeof p === "function") {
			return "(" + p.toString() + ")";
		} else if (typeof p === "bigint") {
			return supportsBigIntLiteral ? `${p}n` : `BigInt("${p}")`;
		} else {
			// assume `p` is a valid JSON value
			return p;
		}
	};
	const normalizeObject = (define: CodeValue): NormalizedCodeValue => {
		if (Array.isArray(define)) {
			return define.map(normalizeObject);
		} else if (define instanceof RegExp) {
			return normalizePrimitive(define);
		} else if (define && typeof define === "object") {
			const keys = Object.keys(define);
			return Object.fromEntries(keys.map(k => [k, normalizeObject(define[k])]));
		} else {
			return normalizePrimitive(define);
		}
	};
	return normalizeObject(define);
};

type CodeValue = RecursiveArrayOrRecord<CodeValuePrimitive>;
type CodeValuePrimitive =
	| null
	| undefined
	| RegExp
	| Function
	| string
	| number
	| boolean
	| bigint
	| undefined;
type NormalizedCodeValuePrimitive = null | string | number | boolean;
type NormalizedCodeValue = RecursiveArrayOrRecord<NormalizedCodeValuePrimitive>;

type RecursiveArrayOrRecord<T> =
	| { [index: string]: RecursiveArrayOrRecord<T> }
	| Array<RecursiveArrayOrRecord<T>>
	| T;
