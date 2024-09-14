"use strict";

const HTMLGeneratorPlugin = require("../../helpers/html-generator-plugin");

module.exports = [
	{
		target: "web",
		name: "one",
		mode: "development",
		context: __dirname,
		entry: "./one.js",
		stats: "none",
		output: {
			path: "/",
			filename: "one-[name].js"
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
		target: "web",
		name: "two",
		mode: "development",
		context: __dirname,
		entry: "./two.js",
		stats: "none",
		output: {
			path: "/",
			filename: "two-[name].js"
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
