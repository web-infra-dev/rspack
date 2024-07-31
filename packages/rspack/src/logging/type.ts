/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/logging/createConsoleLogger.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { FilterTypes } from "../config";

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
