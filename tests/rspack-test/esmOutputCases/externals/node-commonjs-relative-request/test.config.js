const fs = require("fs");
const path = require("path");

function readOutputs(directory) {
	const outputs = new Map();
	const queue = [directory];

	while (queue.length > 0) {
		const current = queue.pop();
		for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
			const file = path.join(current, entry.name);
			if (entry.isDirectory()) {
				queue.push(file);
			} else if (file.endsWith(".mjs")) {
				outputs.set(
					path.relative(directory, file),
					fs.readFileSync(file, "utf-8")
				);
			}
		}
	}

	return outputs;
}

module.exports = {
	beforeExecute(options) {
		if (Array.isArray(options)) {
			options = options[0];
		}
		fs.copyFileSync(
			path.join(options.context, "external.cjs"),
			path.join(options.output.path, "external.cjs")
		);
		fs.cpSync(
			path.join(options.context, "packages", "root"),
			path.join(options.output.path, "node_modules", "shadow-pkg"),
			{ recursive: true }
		);
		fs.cpSync(
			path.join(options.context, "packages", "chunk"),
			path.join(options.output.path, "chunks", "node_modules", "shadow-pkg"),
			{ recursive: true }
		);
	},
	afterExecute(options) {
		const outputs = readOutputs(options.output.path);
		const source = [...outputs.values()].join("\n");
		const lazySource = outputs.get(path.join("chunks", "lazy.mjs"));
		const relativeExternalSource = [...outputs.values()].find(output =>
			output.includes('external "./external.cjs"')
		);
		const packageExternalSource = [...outputs.values()].find(output =>
			output.includes('external "shadow-pkg"')
		);

		expect(lazySource).toBeDefined();
		expect(relativeExternalSource).toBeDefined();
		expect(packageExternalSource).toBeDefined();
		expect(source).toContain('external "./external.cjs"');
		expect(source).toContain('external "shadow-pkg"');
		expect(relativeExternalSource).toMatch(
			/__rspack_createRequire_require\s*\(\s*["']\.\/external\.cjs["']\s*\)/
		);
		expect(packageExternalSource).toMatch(
			/__rspack_createRequire_require\s*\(\s*["']shadow-pkg["']\s*\)/
		);
		expect(lazySource).not.toMatch(
			/__rspack_createRequire_require\s*\(\s*["'](?:\.\/external\.cjs|shadow-pkg)["']\s*\)/
		);
		expect(lazySource).toMatch(
			/(?:__webpack_require__|__rspack_context\.r|rspackRequire)\s*\(\s*[^)]*["']relative-external["']/
		);
		expect(lazySource).toMatch(
			/(?:__webpack_require__|__rspack_context\.r|rspackRequire)\s*\(\s*[^)]*["']package-external["']/
		);
	}
};
