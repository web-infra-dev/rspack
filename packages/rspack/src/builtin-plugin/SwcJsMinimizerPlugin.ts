import {
	BuiltinPluginName,
	RawExtractComments,
	RawSwcJsMinimizerRspackPluginOptions
} from "@rspack/binding";

import { create } from "./base";

type MinifyCondition = string | RegExp;
type MinifyConditions = MinifyCondition | MinifyCondition[];

type ExtractCommentsCondition = boolean | RegExp;
type ExtractCommentsBanner = string | boolean;
// type ExtractFilename = string;
type ExtractCommentsObject = {
	condition?: ExtractCommentsCondition | undefined;
	banner?: ExtractCommentsBanner | undefined;
	// filename?: ExtractFilename | undefined
};
type ExtractCommentsOptions = ExtractCommentsCondition | ExtractCommentsObject;

export type SwcJsMinimizerRspackPluginOptions = {
	/**
	 * @deprecated Deprecated, move to `compress.passes`
	 */
	passes?: number;
	/**
	 * @deprecated Deprecated, move to `compress.drop_console`
	 */
	dropConsole?: boolean;
	/**
	 * @deprecated Deprecated, move to `compress.pure_funcs`
	 */
	pureFuncs?: Array<string>;
	/**
	 * @deprecated Deprecated, move to `mangle.keep_classnames`
	 */
	keepClassNames?: boolean;
	/**
	 * @deprecated Deprecated, move to `mangle.keep_fnames`
	 */
	keepFnNames?: boolean;
	/**
	 * @deprecated Deprecated, move to `format.comments`
	 */
	comments?: false | "all" | "some";
	/**
	 * @deprecated Deprecated, move to `format.ascii_only`
	 */
	asciiOnly?: boolean;
	extractComments?: ExtractCommentsOptions | undefined;
	compress?: TerserCompressOptions | boolean;
	mangle?: TerserMangleOptions | boolean;
	format?: JsFormatOptions & ToSnakeCaseProperties<JsFormatOptions>;
	module?: boolean;

	test?: MinifyConditions;
	exclude?: MinifyConditions;
	include?: MinifyConditions;
};

/**
 * @example ToSnakeCase<'indentLevel'> == 'indent_level'
 */
type ToSnakeCase<T extends string> = T extends `${infer A}${infer B}`
	? `${A extends Lowercase<A> ? A : `_${Lowercase<A>}`}${ToSnakeCase<B>}`
	: T;
/**
 * @example ToSnakeCaseProperties\<{indentLevel: 3\}\> == {indent_level: 3\}
 */
type ToSnakeCaseProperties<T> = {
	[K in keyof T as K extends string ? ToSnakeCase<K> : K]: T[K];
};

export interface JsFormatOptions {
	/**
	 * Currently noop.
	 * @default false
	 * @alias ascii_only
	 */
	asciiOnly?: boolean;
	/**
	 * Currently noop.
	 * @default false
	 */
	beautify?: boolean;
	/**
	 * Currently noop.
	 * @default false
	 */
	braces?: boolean;
	/**
	 * - `false`: removes all comments
	 * - `'some'`: preserves some comments
	 * - `'all'`: preserves all comments
	 * @default false
	 */
	comments?: false | "some" | "all";
	/**
	 * Currently noop.
	 * @default 5
	 */
	ecma?: TerserEcmaVersion;
	/**
	 * Currently noop.
	 * @alias indent_level
	 */
	indentLevel?: number;
	/**
	 * Currently noop.
	 * @alias indent_start
	 */
	indentStart?: number;
	inlineScript?: number;
	/**
	 * Currently noop.
	 * @alias keep_numbers
	 */
	keepNumbers?: number;
	/**
	 * Currently noop.
	 * @alias keep_quoted_props
	 */
	keepQuotedProps?: boolean;
	/**
	 * Currently noop.
	 * @alias max_line_len
	 */
	maxLineLen?: number | false;
	/**
	 * Currently noop.
	 */
	preamble?: string;
	/**
	 * Currently noop.
	 * @alias quote_keys
	 */
	quoteKeys?: boolean;
	/**
	 * Currently noop.
	 * @alias quote_style
	 */
	quoteStyle?: boolean;
	/**
	 * Currently noop.
	 * @alias preserve_annotations
	 */
	preserveAnnotations?: boolean;
	/**
	 * Currently noop.
	 */
	safari10?: boolean;
	/**
	 * Currently noop.
	 */
	semicolons?: boolean;
	/**
	 * Currently noop.
	 */
	shebang?: boolean;
	/**
	 * Currently noop.
	 */
	webkit?: boolean;
	/**
	 * Currently noop.
	 * @alias wrap_iife
	 */
	wrapIife?: boolean;
	/**
	 * Currently noop.
	 * @alias wrap_func_args
	 */
	wrapFuncArgs?: boolean;
}

export type TerserEcmaVersion = 5 | 2015 | 2016 | string | number;
export interface TerserCompressOptions {
	arguments?: boolean;
	arrows?: boolean;
	booleans?: boolean;
	booleans_as_integers?: boolean;
	collapse_vars?: boolean;
	comparisons?: boolean;
	computed_props?: boolean;
	conditionals?: boolean;
	dead_code?: boolean;
	defaults?: boolean;
	directives?: boolean;
	drop_console?: boolean;
	drop_debugger?: boolean;
	ecma?: TerserEcmaVersion;
	evaluate?: boolean;
	expression?: boolean;
	global_defs?: any;
	hoist_funs?: boolean;
	hoist_props?: boolean;
	hoist_vars?: boolean;
	ie8?: boolean;
	if_return?: boolean;
	inline?: 0 | 1 | 2 | 3;
	join_vars?: boolean;
	keep_classnames?: boolean;
	keep_fargs?: boolean;
	keep_fnames?: boolean;
	keep_infinity?: boolean;
	loops?: boolean;
	negate_iife?: boolean;
	passes?: number;
	properties?: boolean;
	pure_getters?: any;
	pure_funcs?: string[];
	reduce_funcs?: boolean;
	reduce_vars?: boolean;
	sequences?: any;
	side_effects?: boolean;
	switches?: boolean;
	top_retain?: any;
	toplevel?: any;
	typeofs?: boolean;
	unsafe?: boolean;
	unsafe_passes?: boolean;
	unsafe_arrows?: boolean;
	unsafe_comps?: boolean;
	unsafe_function?: boolean;
	unsafe_math?: boolean;
	unsafe_symbols?: boolean;
	unsafe_methods?: boolean;
	unsafe_proto?: boolean;
	unsafe_regexp?: boolean;
	unsafe_undefined?: boolean;
	unused?: boolean;
	const_to_let?: boolean;
	module?: boolean;
}
export interface TerserMangleOptions {
	props?: TerserManglePropertiesOptions;
	toplevel?: boolean;
	keep_classnames?: boolean;
	keep_fnames?: boolean;
	keep_private_props?: boolean;
	ie8?: boolean;
	safari10?: boolean;
	reserved?: string[];
}
export interface TerserManglePropertiesOptions {}

function getRawCompressOptions(options?: SwcJsMinimizerRspackPluginOptions) {
	function _inner(): TerserCompressOptions | boolean {
		const _default = {
			passes: options?.passes ?? 1,
			pure_funcs: options?.pureFuncs ?? [],
			drop_console: options?.dropConsole ?? false
		} satisfies TerserCompressOptions;

		if (options?.compress === true) {
			return _default;
		}

		if (options?.compress === false) {
			return false;
		}

		if (options?.compress && typeof options.compress === "object") {
			return {
				// TODO: deprecate default merging in 0.4
				..._default,
				...options.compress
			};
		}

		return _default;
	}

	let inner = _inner();

	return typeof inner === "boolean" ? inner : JSON.stringify(inner);
}

function getRawMangleOptions(options?: SwcJsMinimizerRspackPluginOptions) {
	function _inner(): TerserMangleOptions | boolean {
		const _default = {
			keep_classnames: options?.keepClassNames ?? false,
			keep_fnames: options?.keepFnNames ?? false
		} satisfies TerserMangleOptions;

		if (options?.mangle === true) {
			return _default;
		}

		if (options?.mangle === false) {
			return false;
		}

		if (options?.mangle && typeof options.mangle === "object") {
			return {
				// TODO: deprecate default merging in 0.4
				..._default,
				...options.mangle
			};
		}

		return _default;
	}

	let inner = _inner();

	return typeof inner === "boolean" ? inner : JSON.stringify(inner);
}

function getRawFormatOptions(options?: SwcJsMinimizerRspackPluginOptions) {
	function _inner() {
		const _default = {
			comments: options?.comments ? options?.comments : false,
			asciiOnly: options?.asciiOnly ?? false
		} satisfies SwcJsMinimizerRspackPluginOptions["format"];

		if (options?.format && typeof options.format === "object") {
			// TODO: deprecate default merging in 0.4
			return {
				..._default,
				...options.format
			};
		}

		return _default;
	}

	return JSON.stringify(_inner());
}

function isObject(value: any): value is Object {
	const type = typeof value;

	return value != null && (type === "object" || type === "function");
}

function getRawExtractCommentsOptions(
	extractComments?: ExtractCommentsOptions
): RawExtractComments | undefined {
	const conditionStr = (condition?: ExtractCommentsCondition): string => {
		if (typeof condition === "undefined" || condition === true) {
			// copied from terser-webpack-plugin
			return "@preserve|@lic|@cc_on|^\\**!";
		} else if (condition === false) {
			throw Error("unreachable");
		} else {
			// FIXME: flags
			return condition.source;
		}
	};
	if (typeof extractComments === "boolean") {
		if (!extractComments) {
			return undefined;
		} else {
			const res = {
				condition: conditionStr(extractComments)
			};
			return res;
		}
	} else if (extractComments instanceof RegExp) {
		const res = {
			condition: extractComments.source
		};
		return res;
	} else if (isObject(extractComments)) {
		if (extractComments.condition === false) {
			return undefined;
		} else {
			const res = {
				condition: conditionStr(extractComments.condition),
				banner: extractComments.banner
			};
			return res;
		}
	} else {
		return undefined;
	}
}

export const SwcJsMinimizerRspackPlugin = create(
	BuiltinPluginName.SwcJsMinimizerRspackPlugin,
	(
		options?: SwcJsMinimizerRspackPluginOptions
	): RawSwcJsMinimizerRspackPluginOptions => {
		return {
			extractComments: getRawExtractCommentsOptions(options?.extractComments),
			compress: getRawCompressOptions(options),
			mangle: getRawMangleOptions(options),
			format: getRawFormatOptions(options),
			module: options?.module,
			test: options?.test,
			include: options?.include,
			exclude: options?.exclude
		};
	},
	"compilation"
);
