"use strict";

const toml = require("toml");

/** @typedef {import("@rspack/core").ParserOptionsByModuleTypeKnown} ParserOptionsByModuleTypeKnown */

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		mode: "development",
		module: {
			rules: [
				{
					test: /\.toml$/,
					type: "json",
					/** @type {ParserOptionsByModuleTypeKnown['json']} */
					parser: {
						parse(input) {
							// eslint-disable-next-line prefer-rest-params
							expect(arguments).toHaveLength(1);
							return toml.parse(input);
						}
					}
				}
			]
		}
	}
];
