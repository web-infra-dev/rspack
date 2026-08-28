"use strict";

const fs = require("fs");
const path = require("path");
const url = require("url");

module.exports = {
	findBundle(_, options) {
		const chunks = fs
			.readdirSync(path.join(options.output.path, "chunks"))
			.map(file => `./chunks/${file}`);
		return [...chunks, "./bundle0.js"];
	},
	moduleScope(scope, _, options) {
		const warnings = (scope.window.__wasmStreamingFallbackWarnings ??= []);
		const sandboxConsole = Object.create(console);
		sandboxConsole.warn = (...args) => warnings.push(args);
		scope.window.console = sandboxConsole;
		scope.fetch = resource =>
			new Promise((resolve, reject) => {
				const file = /^file:/i.test(resource)
					? url.fileURLToPath(resource)
					: path.join(options.output.path, path.basename(resource));

				fs.readFile(file, (err, data) => {
					if (err) {
						reject(err);
						return;
					}

					resolve(
						new Response(data, {
							headers: { "Content-Type": "application/octet-stream" }
						})
					);
				});
			});
	}
};
