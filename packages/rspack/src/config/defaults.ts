import { RspackOptions, RspackOptionsNormalized } from "..";
import { getDefaultTarget } from "./target";
const D = <T, P extends keyof T>(obj, prop, value) => {
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

export function applyRspackOptionsDefaults(options: RspackOptionsNormalized) {
	F(options, "context", () => process.cwd());
	/** @todo  */
	F(options, "target", () => {
		return getDefaultTarget(options.context);
	});
	const { mode } = options;
	const development = mode === "development";
	const production = mode === "production" || !mode;
	F(options, "devtool", () => (development ? "eval" : ""));
}
