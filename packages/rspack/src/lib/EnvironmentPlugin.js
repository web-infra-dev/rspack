/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Authors Simen Brekken @simenbrekken, Einar Löve @einarlove
*/

"use strict";

import { DefinePlugin } from "../builtin-plugin";

const WebpackError = require("./WebpackError");

// /** @typedef {import("./Compiler")} Compiler */
// /** @typedef {import("./DefinePlugin").CodeValue} CodeValue */
/** @typedef {any} Compiler */
/** @typedef {any} CodeValue */

class EnvironmentPlugin {
	// @ts-expect-error
	constructor(...keys) {
		if (keys.length === 1 && Array.isArray(keys[0])) {
			this.keys = keys[0];
			this.defaultValues = {};
		} else if (keys.length === 1 && keys[0] && typeof keys[0] === "object") {
			this.keys = Object.keys(keys[0]);
			this.defaultValues = keys[0];
		} else {
			this.keys = keys;
			this.defaultValues = {};
		}
	}

	/**
	 * Apply the plugin
	 * @param {Compiler} compiler the compiler instance
	 * @returns {void}
	 */
	apply(compiler) {
		/** @type {Record<string, CodeValue>} */
		const definitions = {};
		for (const key of this.keys) {
			const value =
				process.env[key] !== undefined
					? process.env[key]
					: this.defaultValues[key];

			if (value === undefined) {
				// @ts-expect-error
				compiler.hooks.thisCompilation.tap("EnvironmentPlugin", compilation => {
					const error = new WebpackError(
						`EnvironmentPlugin - ${key} environment variable is undefined.\n\n` +
							"You can pass an object with default values to suppress this warning.\n" +
							"See https://webpack.js.org/plugins/environment-plugin for example."
					);

					error.name = "EnvVariableNotDefinedError";
					compilation.errors.push(error);
				});
			}

			definitions[`process.env.${key}`] =
				value === undefined ? "undefined" : JSON.stringify(value);
		}
		new DefinePlugin(definitions).apply(compiler);
	}
}

export { EnvironmentPlugin };
