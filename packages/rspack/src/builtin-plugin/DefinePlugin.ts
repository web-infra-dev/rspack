import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export type DefinePluginOptions = Record<string, CodeValue>;

/**
 * Options for RuntimeValue to specify dependencies and version
 */
export interface RuntimeValueOptions {
	/** Files that should be watched for changes */
	fileDependencies?: string[];
	/** Directories that should be watched for changes */
	contextDependencies?: string[];
	/** Files that are expected but not found */
	missingDependencies?: string[];
	/** Dependencies that affect the entire build */
	buildDependencies?: string[];
	/** Version string or function to get version, used for caching */
	version?: string | (() => string);
}

/**
 * Function type for dynamic value computation at compile time
 *
 * Note: The `module` parameter is currently not supported in rspack.
 * It will always be undefined. This is a known limitation compared to webpack.
 */
export type RuntimeValueFn = (arg: {
	/** The current module being processed (not supported in rspack, always undefined) */
	module?: any;
	/** The key in DefinePlugin options */
	key?: string;
	/** Version string from RuntimeValueOptions */
	version?: string;
}) => CodeValuePrimitive;

/**
 * Represents a value that is computed dynamically at compile time
 * Compatible with webpack's DefinePlugin.runtimeValue
 */
export class RuntimeValue {
	/** Function to compute the value */
	public fn: RuntimeValueFn;
	/** Options for dependency tracking and caching */
	public options: boolean | string[] | RuntimeValueOptions;

	constructor(
		fn: RuntimeValueFn,
		options: boolean | string[] | RuntimeValueOptions = true
	) {
		this.fn = fn;
		this.options = options;
	}
}

const DefinePluginImpl = create(
	BuiltinPluginName.DefinePlugin,
	function (define: DefinePluginOptions): NormalizedCodeValue {
		const supportsBigIntLiteral =
			this.options.output.environment?.bigIntLiteral ?? false;
		const processedDefine = processDefineOptions(define, this);
		const normalized = normalizeValue(processedDefine, supportsBigIntLiteral);

		return normalized;
	},
	"compilation"
);

export const DefinePlugin = Object.assign(DefinePluginImpl, {
	/**
	 * Create a RuntimeValue for dynamic compile-time value computation
	 * @param fn - Function to compute the value
	 * @param options - true to cache, string[] for file dependencies, or full options object
	 * @returns RuntimeValue instance
	 */
	runtimeValue: function (
		fn: RuntimeValueFn,
		options: boolean | string[] | RuntimeValueOptions = true
	): RuntimeValue {
		return new RuntimeValue(fn, options);
	}
});

/**
 * Process DefinePlugin options, handling both static values and RuntimeValues
 * @param define - The DefinePlugin options to process
 * @param compiler - The compiler context with compilation info
 * @returns Processed define options with RuntimeValues executed or preserved for later
 */
function processDefineOptions(
	define: DefinePluginOptions,
	compiler: any
): Record<string, CodeValue> {
	const result: Record<string, CodeValue> = {};
	const compilation = compiler.compilation;

	for (const [key, value] of Object.entries(define)) {
		if (value instanceof RuntimeValue) {
			// For now, execute runtime value without module context
			// TODO: In the future, we should delay execution until parse time
			// when module context is available
			const context = {
				module: undefined, // Module context requires delayed execution
				key,
				version:
					typeof value.options === "object" &&
					!Array.isArray(value.options) &&
					value.options.version
						? typeof value.options.version === "function"
							? value.options.version()
							: value.options.version
						: undefined
			};

			try {
				// Note: This executes at compilation time, not parse time
				// Module-specific values cannot be supported with current architecture
				result[key] = value.fn(context);

				// Add dependencies to the compilation for watch mode
				if (compilation) {
					if (Array.isArray(value.options)) {
						// Legacy format: array of file dependencies
						value.options.forEach(dep => compilation.fileDependencies.add(dep));
					} else if (typeof value.options === "object") {
						// Full options object with different dependency types
						value.options.fileDependencies?.forEach(dep =>
							compilation.fileDependencies.add(dep)
						);
						value.options.contextDependencies?.forEach(dep =>
							compilation.contextDependencies.add(dep)
						);
						value.options.missingDependencies?.forEach(dep =>
							compilation.missingDependencies.add(dep)
						);
						if (value.options.buildDependencies) {
							// buildDependencies affect the whole build
							value.options.buildDependencies.forEach(dep => {
								if (!compilation.buildDependencies.has(dep)) {
									compilation.buildDependencies.add(dep);
								}
							});
						}
					}
				}
			} catch (err) {
				// If runtime value execution fails, use undefined
				console.error(
					`DefinePlugin runtime value error for key "${key}":`,
					err
				);
				result[key] = undefined;
			}
		} else if (
			typeof value === "object" &&
			value !== null &&
			!(value instanceof RegExp) &&
			!(value instanceof Function) &&
			!Array.isArray(value)
		) {
			// Recursively process nested objects
			result[key] = processDefineOptions(
				value as DefinePluginOptions,
				compiler
			);
		} else {
			// Static value: string, number, boolean, array, etc.
			result[key] = value;
		}
	}

	return result;
}

const normalizeValue = (
	define: DefinePluginOptions,
	supportsBigIntLiteral: boolean
): NormalizedCodeValue => {
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
			return Object.fromEntries(
				keys.map(k => [
					k,
					normalizeObject((define as Record<string, CodeValue>)[k])
				])
			);
		}
		return normalizePrimitive(define);
	};
	return normalizeObject(define);
};

type CodeValue = RecursiveArrayOrRecord<CodeValuePrimitive | RuntimeValue>;
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
	| Array<RecursiveArrayOrRecord<T>>
	| T;
