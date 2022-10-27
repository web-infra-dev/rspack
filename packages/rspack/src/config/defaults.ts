import { RspackOptions, RspackOptionsNormalized } from "..";
import path from "path";
import { getDefaultTarget } from "./target";
import { ResolvedOutput } from "./output";
const D = <T, P extends keyof T>(obj: T, prop: P, value: T[P]) => {
	if (obj[prop] === undefined) {
		obj[prop] = value;
	}
};
const F = <T, P extends keyof T>(obj: T, prop: P, factory: () => T[P]) => {
	if (obj[prop] === undefined) {
		obj[prop] = factory();
	}
};
export function applyRspackOptionsBaseDefaults(
	options: RspackOptionsNormalized
) {
	F(options, "context", () => process.cwd());
	applyInfrastructureLoggingDefaults(options.infrastructureLogging);
}

/**
 * @param {InfrastructureLogging} infrastructureLogging options
 * @returns {void}
 */
const applyInfrastructureLoggingDefaults = infrastructureLogging => {
	F(infrastructureLogging, "stream", () => process.stderr);
	const tty =
		/** @type {any} */ infrastructureLogging.stream.isTTY &&
		process.env.TERM !== "dumb";
	D(infrastructureLogging, "level", "info");
	D(infrastructureLogging, "debug", false);
	D(infrastructureLogging, "colors", tty);
	D(infrastructureLogging, "appendOnly", !tty);
};
const applyOutputDefault = (output: ResolvedOutput) => {
	D(output, "hashFunction", "xxhash64");
	F(output, "path", () => path.resolve(process.cwd(), "dist"));
	return output;
};
export function applyRspackOptionsDefaults(options: RspackOptionsNormalized) {
	F(options, "context", () => process.cwd());
	/** @todo  */
	F(options, "target", () => {
		return getDefaultTarget(options.context);
	});
	const { mode, context } = options;
	const development = mode === "development";
	const production = mode === "production" || !mode;
	F(options, "devtool", () => (development ? "eval" : ""));
	applyOutputDefault(options.output);
	D(options.builtins, "minify", production);
}
