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
		}
		if (Object.is(p, -0)) {
			return "-0";
		}
		if (p instanceof RegExp) {
			return p.toString();
		}
		if (typeof p === "function") {
			return `(${p.toString()})`;
		}
		if (typeof p === "bigint") {
			return supportsBigIntLiteral ? `${p}n` : `BigInt("${p}")`;
		}
		// assume `p` is a valid JSON value
		return p;
	};
	const normalizeObject = (define: CodeValue): NormalizedCodeValue => {
		if (Array.isArray(define)) {
			return define.map(normalizeObject);
		}
		if (define instanceof RegExp) {
			return normalizePrimitive(define);
		}
		if (define && typeof define === "object") {
			const keys = Object.keys(define);
			return Object.fromEntries(keys.map(k => [k, normalizeObject(define[k])]));
		}
		return normalizePrimitive(define);
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
	| bigint;
type NormalizedCodeValuePrimitive = null | string | number | boolean;
type NormalizedCodeValue = RecursiveArrayOrRecord<NormalizedCodeValuePrimitive>;

type RecursiveArrayOrRecord<T> =
	| { [index: string]: RecursiveArrayOrRecord<T> }
	| RecursiveArrayOrRecord<T>[]
	| T;
