/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Tobias Koppers @sokra
*/

const SyncBailHook = require("tapable/lib/SyncBailHook");
const { Logger } = require("./Logger");
const createConsoleLogger = require("./createConsoleLogger");

/** @type {createConsoleLogger.LoggerOptions} */
let currentDefaultLoggerOptions = {
	level: "info",
	debug: false,
	console
};
let currentDefaultLogger = createConsoleLogger(currentDefaultLoggerOptions);

/**
 * @param {string} name name of the logger
 * @returns {Logger} a logger
 */
export const getLogger = (name) => {
	return new Logger(
		(type, args) => {
			if (exports.hooks.log.call(name, type, args) === undefined) {
				currentDefaultLogger(name, type, args);
			}
		},
		(childName) => exports.getLogger(`${name}/${childName}`)
	);
};

/**
 * @param {createConsoleLogger.LoggerOptions} options new options, merge with old options
 * @returns {void}
 */
export const configureDefaultLogger = (options) => {
	Object.assign(currentDefaultLoggerOptions, options);
	currentDefaultLogger = createConsoleLogger(currentDefaultLoggerOptions);
};

export const hooks = {
	log: new SyncBailHook(["origin", "type", "args"])
};
