const fs = require("node:fs");
const path = require("node:path");
const { CopyRspackPlugin, Stats } = require("@rspack/core");

function write(root, filename, content) {
	const file = path.join(root, filename);
	fs.mkdirSync(path.dirname(file), { recursive: true });
	fs.writeFileSync(file, content);
	return file;
}

function remove(root, filename) {
	const file = path.join(root, filename);
	fs.rmSync(file);
	return file;
}

function createProject(context, name, patterns, cache = true) {
	const root = context.getDist(name);
	fs.rmSync(root, { recursive: true, force: true });
	write(root, "src/index.js", "module.exports = 'initial';\n");

	return {
		root,
		options: {
			context: root,
			mode: "development",
			target: "node",
			devtool: false,
			cache,
			incremental: true,
			entry: "./src/index.js",
			output: { path: path.join(root, "dist"), filename: "main.js" },
			plugins: [new CopyRspackPlugin({ patterns })]
		}
	};
}

function useRealOutputFileSystem(_context, compiler) {
	compiler.outputFileSystem = fs;
}

function compile(compiler) {
	return new Promise((resolve, reject) => {
		compiler.run((error, stats) => {
			if (error) return reject(error);
			if (stats.hasErrors()) {
				return reject(new Error(stats.toString({ all: false, errors: true })));
			}
			resolve(compiler._lastCompilation);
		});
	});
}

function rebuild(compiler, modifiedFiles = [], removedFiles = []) {
	return new Promise((resolve, reject) => {
		compiler.__internal__rebuild(
			new Set(modifiedFiles),
			new Set(removedFiles),
			error => {
				if (error) return reject(error);
				const compilation = compiler._lastCompilation;
				if (compilation.errors.length > 0) {
					return reject(
						new Error(new Stats(compilation).toString({ all: false, errors: true }))
					);
				}
				resolve(compilation);
			}
		);
	});
}

function rebuildAllowErrors(compiler, modifiedFiles = [], removedFiles = []) {
	return new Promise((resolve, reject) => {
		compiler.__internal__rebuild(
			new Set(modifiedFiles),
			new Set(removedFiles),
			error => {
				if (error) return reject(error);
				resolve(compiler._lastCompilation);
			}
		);
	});
}

function asset(compilation, filename) {
	return compilation.getAsset(filename)?.source.source().toString();
}

function reusedPatterns(compilation) {
	const logging = new Stats(compilation).toJson({
		all: false,
		logging: false,
		loggingDebug: [/CopyRspackPlugin/]
	}).logging;

	const entries = Object.values(logging || {})
		.flatMap(group => group.entries || [])
		.map(entry => entry.message || "");
	const summary = entries.find(message =>
		message.startsWith("copy pattern cache: ")
	);
	const hits = summary?.match(/\((\d+)\/\d+\)$/);
	return hits ? Number(hits[1]) : 0;
}

module.exports = {
	asset,
	compile,
	createProject,
	rebuild,
	rebuildAllowErrors,
	remove,
	reusedPatterns,
	useRealOutputFileSystem,
	write
};
