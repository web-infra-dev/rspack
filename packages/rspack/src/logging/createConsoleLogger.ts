/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/logging/createConsoleLogger.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { FilterItemTypes, FilterTypes } from "../config";
import { LogType, type LogTypeEnum } from "./Logger";

export type FilterFunction = (ident: string) => boolean;

export type LoggerConsole = {
	clear: () => void;
	trace: () => void;
	info: (...args: any[]) => void;
	log: (...args: any[]) => void;
	warn: (...args: any[]) => void;
	error: (...args: any[]) => void;
	debug?: (...args: any[]) => void;
	group?: (...args: any[]) => void;
	groupCollapsed?: (...args: any[]) => void;
	groupEnd?: (...args: any[]) => void;
	status?: (...args: any[]) => void;
	profile?: (...args: any[]) => void;
	profileEnd?: (...args: any[]) => void;
	logTime?: (...args: any[]) => void;
};

export type LoggerOptions = {
	level: "none" | "error" | "warn" | "info" | "log" | "verbose" | boolean;
	debug: FilterTypes | boolean;
	console: LoggerConsole;
};

const filterToFunction = (
	item: FilterItemTypes
): FilterFunction | undefined => {
	if (typeof item === "string") {
		const regExp = new RegExp(
			`[\\\\/]${item.replace(
				// eslint-disable-next-line no-useless-escape
				/[-[\]{}()*+?.\\^$|]/g,
				"\\$&"
			)}([\\\\/]|$|!|\\?)`
		);
		return ident => regExp.test(ident);
	}
	if (item && typeof item === "object" && typeof item.test === "function") {
		return ident => item.test(ident);
	}
	if (typeof item === "function") {
		return item;
	}
	if (typeof item === "boolean") {
		return () => item;
	}
};

const LogLevel = {
	none: 6,
	false: 6,
	error: 5,
	warn: 4,
	info: 3,
	log: 2,
	true: 2,
	verbose: 1
};

const createConsoleLogger = ({
	level = "info",
	debug = false,
	console
}: LoggerOptions) => {
	const debugFilters =
		typeof debug === "boolean"
			? [() => debug]
			: ([] as FilterItemTypes[]).concat(debug).map(filterToFunction);
	const loglevel = LogLevel[`${level}`] || 0;

	const logger = (name: string, type: LogTypeEnum, args: any[]): void => {
		const labeledArgs = () => {
			if (Array.isArray(args)) {
				if (args.length > 0 && typeof args[0] === "string") {
					return [`[${name}] ${args[0]}`, ...args.slice(1)];
				}
				return [`[${name}]`, ...args];
			}
			return [];
		};
		const debug = debugFilters.some(f => f!(name));
		switch (type) {
			case LogType.debug:
				if (!debug) return;

				if (typeof console.debug === "function") {
					console.debug(...labeledArgs());
				} else {
					console.log(...labeledArgs());
				}
				break;
			case LogType.log:
				if (!debug && loglevel > LogLevel.log) return;
				console.log(...labeledArgs());
				break;
			case LogType.info:
				if (!debug && loglevel > LogLevel.info) return;
				console.info(...labeledArgs());
				break;
			case LogType.warn:
				if (!debug && loglevel > LogLevel.warn) return;
				console.warn(...labeledArgs());
				break;
			case LogType.error:
				if (!debug && loglevel > LogLevel.error) return;
				console.error(...labeledArgs());
				break;
			case LogType.trace:
				if (!debug) return;
				console.trace();
				break;
			// biome-ignore lint/suspicious/noFallthroughSwitchClause: This case is falling through to the next case.
			case LogType.groupCollapsed:
				if (!debug && loglevel > LogLevel.log) return;
				if (!debug && loglevel > LogLevel.verbose) {
					if (typeof console.groupCollapsed === "function") {
						console.groupCollapsed(...labeledArgs());
					} else {
						console.log(...labeledArgs());
					}
					break;
				}
			// falls through
			case LogType.group:
				if (!debug && loglevel > LogLevel.log) return;

				if (typeof console.group === "function") {
					console.group(...labeledArgs());
				} else {
					console.log(...labeledArgs());
				}
				break;
			case LogType.groupEnd:
				if (!debug && loglevel > LogLevel.log) return;

				if (typeof console.groupEnd === "function") {
					console.groupEnd();
				}
				break;
			case LogType.time: {
				if (!debug && loglevel > LogLevel.log) return;
				const ms = args[1] * 1000 + args[2] / 1000000;
				const msg = `[${name}] ${args[0]}: ${ms} ms`;
				if (typeof console.logTime === "function") {
					console.logTime(msg);
				} else {
					console.log(msg);
				}
				break;
			}
			case LogType.profile:
				if (typeof console.profile === "function") {
					console.profile(...labeledArgs());
				}
				break;
			case LogType.profileEnd:
				if (typeof console.profileEnd === "function") {
					console.profileEnd(...labeledArgs());
				}
				break;
			case LogType.clear:
				if (!debug && loglevel > LogLevel.log) return;

				if (typeof console.clear === "function") {
					console.clear();
				}
				break;
			case LogType.status:
				if (!debug && loglevel > LogLevel.info) return;
				if (typeof console.status === "function") {
					if (args.length === 0) {
						console.status();
					} else {
						console.status(...labeledArgs());
					}
				} else {
					if (args.length !== 0) {
						console.info(...labeledArgs());
					}
				}
				break;
			default:
				throw new Error(`Unexpected LogType ${type}`);
		}
	};
	return logger;
};

export { createConsoleLogger };
