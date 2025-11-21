const path = require("node:path");
const fs = require("node:fs");

class ShouldRebuildPlugin {
	constructor() {
		this.compileCount = 0;
	}
	apply(compiler) {
		const targetFile = path.resolve(compiler.context, "./index.js");

		compiler.hooks.done.tap(ShouldRebuildPlugin.name, _ => {
			// After first compilation, touch the file to trigger a rebuild
			if (this.compileCount === 0) {
				const now = new Date();
				fs.utimes(targetFile, now, now, err => {
					if (err) {
						console.error("Error updating file timestamps:", err);
						return;
					}
					// Touch file to trigger rebuild
				});
			}
			this.compileCount++;
		});
	}
}

/**
 * @type {import('@rspack/core').Configuration}
 */
const config = {
	plugins: [new ShouldRebuildPlugin()],
	watchOptions: {
		aggregateTimeout: 1000
	}
};

module.exports = config;
