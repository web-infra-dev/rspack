"use strict";

const HTMLGeneratorPlugin = require("../../helpers/html-generator-plugin");

module.exports = [
	{
		name: "browser",
		mode: "development",
		context: __dirname,
		stats: "none",
		entry: "./browser.js",
		output: {
			path: "/",
			filename: "browser.js"
		},
		plugins: [new HTMLGeneratorPlugin()],
		infrastructureLogging: {
			level: "info",
			stream: {
				write: () => {}
			}
		}
	},
	{
		name: "server",
		mode: "development",
		context: __dirname,
		target: "node",
		stats: "none",
		entry: "./server.js",
		output: {
			path: "/",
			filename: "server.js"
		},
		plugins: [new HTMLGeneratorPlugin()],
		infrastructureLogging: {
			level: "info",
			stream: {
				write: () => {}
			}
		}
	}
];
