/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/logging/runtime.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

const SyncBailHook = require("tapable/lib/SyncBailHook");
import { Logger } from "./Logger";
import createConsoleLogger from "./createConsoleLogger";
import type { LoggerOptions } from "./type";

const currentDefaultLoggerOptions: LoggerOptions = {
	level: "info",
	debug: false,
	console
};
let currentDefaultLogger = createConsoleLogger(currentDefaultLoggerOptions);

export const getLogger = (name: string): Logger => {
	return new Logger(
		(type, args) => {
			if (exports.hooks.log.call(name, type, args) === undefined) {
				currentDefaultLogger(name, type, args);
			}
		},
		childName => exports.getLogger(`${name}/${childName}`)
	);
};

export const configureDefaultLogger = (options: LoggerOptions): void => {
	Object.assign(currentDefaultLoggerOptions, options);
	currentDefaultLogger = createConsoleLogger(currentDefaultLoggerOptions);
};

export const hooks = {
	log: new SyncBailHook(["origin", "type", "args"])
};
