/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/logging/Logger.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

export const LogType = Object.freeze({
	error: /** @type {"error"} */ "error", // message, c style arguments
	warn: /** @type {"warn"} */ "warn", // message, c style arguments
	info: /** @type {"info"} */ "info", // message, c style arguments
	log: /** @type {"log"} */ "log", // message, c style arguments
	debug: /** @type {"debug"} */ "debug", // message, c style arguments

	trace: /** @type {"trace"} */ "trace", // no arguments

	group: /** @type {"group"} */ "group", // [label]
	groupCollapsed: /** @type {"groupCollapsed"} */ "groupCollapsed", // [label]
	groupEnd: /** @type {"groupEnd"} */ "groupEnd", // [label]

	profile: /** @type {"profile"} */ "profile", // [profileName]
	profileEnd: /** @type {"profileEnd"} */ "profileEnd", // [profileName]

	time: /** @type {"time"} */ "time", // name, time as [seconds, nanoseconds]

	clear: /** @type {"clear"} */ "clear", // no arguments
	status: /** @type {"status"} */ "status", // message, arguments
	cache: /** @type {"cache"} */ "cache" // [hit, total]
});

export function getLogTypeBitFlag(type: LogTypeEnum) {
	return 1 << Object.values(LogType).findIndex(i => i === type);
}

export function getLogTypesBitFlag(types: LogTypeEnum[]) {
	return types.reduce((acc, cur) => acc | getLogTypeBitFlag(cur), 0);
}

export type LogTypeEnum = (typeof LogType)[keyof typeof LogType];

const LOG_SYMBOL = Symbol("webpack logger raw log method");
const TIMERS_SYMBOL = Symbol("webpack logger times");
const TIMERS_AGGREGATES_SYMBOL = Symbol("webpack logger aggregated times");

export type LogFunction = (type: LogTypeEnum, args: any[]) => void;
export type GetChildLogger = (name: string | (() => string)) => Logger;

export class Logger {
	getChildLogger: GetChildLogger;
	[LOG_SYMBOL]: any;
	[TIMERS_SYMBOL]: any;
	[TIMERS_AGGREGATES_SYMBOL]: any;
	constructor(log: LogFunction, getChildLogger: GetChildLogger) {
		this[LOG_SYMBOL] = log;
		this.getChildLogger = getChildLogger;
	}

	error(...args: any[]) {
		this[LOG_SYMBOL](LogType.error, args);
	}

	warn(...args: any[]) {
		this[LOG_SYMBOL](LogType.warn, args);
	}

	info(...args: any[]) {
		this[LOG_SYMBOL](LogType.info, args);
	}

	log(...args: any[]) {
		this[LOG_SYMBOL](LogType.log, args);
	}

	debug(...args: string[]) {
		this[LOG_SYMBOL](LogType.debug, args);
	}

	assert(assertion: any, ...args: any[]) {
		if (!assertion) {
			this[LOG_SYMBOL](LogType.error, args);
		}
	}

	trace() {
		this[LOG_SYMBOL](LogType.trace, ["Trace"]);
	}

	clear() {
		this[LOG_SYMBOL](LogType.clear);
	}

	status(...args: any[]) {
		this[LOG_SYMBOL](LogType.status, args);
	}

	group(...args: any[]) {
		this[LOG_SYMBOL](LogType.group, args);
	}

	groupCollapsed(...args: any[]) {
		this[LOG_SYMBOL](LogType.groupCollapsed, args);
	}

	groupEnd(...args: any[]) {
		this[LOG_SYMBOL](LogType.groupEnd, args);
	}

	profile(label: any) {
		this[LOG_SYMBOL](LogType.profile, [label]);
	}

	profileEnd(label: any) {
		this[LOG_SYMBOL](LogType.profileEnd, [label]);
	}

	time(label: any) {
		this[TIMERS_SYMBOL] = this[TIMERS_SYMBOL] || new Map();
		this[TIMERS_SYMBOL].set(label, process.hrtime());
	}

	timeLog(label: any) {
		const prev = this[TIMERS_SYMBOL] && this[TIMERS_SYMBOL].get(label);
		if (!prev) {
			throw new Error(`No such label '${label}' for WebpackLogger.timeLog()`);
		}
		const time = process.hrtime(prev);
		this[LOG_SYMBOL](LogType.time, [label, ...time]);
	}

	timeEnd(label: any) {
		const prev = this[TIMERS_SYMBOL] && this[TIMERS_SYMBOL].get(label);
		if (!prev) {
			throw new Error(`No such label '${label}' for WebpackLogger.timeEnd()`);
		}
		const time = process.hrtime(prev);
		this[TIMERS_SYMBOL].delete(label);
		this[LOG_SYMBOL](LogType.time, [label, ...time]);
	}

	timeAggregate(label: any) {
		const prev = this[TIMERS_SYMBOL] && this[TIMERS_SYMBOL].get(label);
		if (!prev) {
			throw new Error(
				`No such label '${label}' for WebpackLogger.timeAggregate()`
			);
		}
		const time = process.hrtime(prev);
		this[TIMERS_SYMBOL].delete(label);
		this[TIMERS_AGGREGATES_SYMBOL] =
			this[TIMERS_AGGREGATES_SYMBOL] || new Map();
		const current = this[TIMERS_AGGREGATES_SYMBOL].get(label);
		if (current !== undefined) {
			if (time[1] + current[1] > 1e9) {
				time[0] += current[0] + 1;
				time[1] = time[1] - 1e9 + current[1];
			} else {
				time[0] += current[0];
				time[1] += current[1];
			}
		}
		this[TIMERS_AGGREGATES_SYMBOL].set(label, time);
	}

	timeAggregateEnd(label: any) {
		if (this[TIMERS_AGGREGATES_SYMBOL] === undefined) return;
		const time = this[TIMERS_AGGREGATES_SYMBOL].get(label);
		if (time === undefined) return;
		this[TIMERS_AGGREGATES_SYMBOL].delete(label);
		this[LOG_SYMBOL](LogType.time, [label, ...time]);
	}
}
