const {
	asset,
	compile,
	createProject,
	rebuild,
	reusedPatterns,
	useRealOutputFileSystem,
	write
} = require("./_copy-plugin-cache");

function patternOrderCase() {
	const patternCount = 64;
	let root;
	let files;
	let winner;

	return {
		description: "should preserve pattern order across cache hits and misses",
		options(context) {
			const patterns = Array.from({ length: patternCount }, (_, index) => ({
				from: `assets/${index}.txt`,
				to: "copied/value.txt",
				toType: "file",
				force: true,
				priority: index % 5
			}));
			const project = createProject(context, "mixed-cache-order", patterns);
			root = project.root;
			files = patterns.map((_, index) =>
				write(root, `assets/${index}.txt`, `${index}\n`)
			);
			winner = Math.floor((patternCount - 5) / 5) * 5 + 4;
			return project.options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			const initial = await compile(compiler);
			expect(asset(initial, "copied/value.txt")).toBe(`${winner}\n`);

			write(root, "assets/0.txt", "first-edited\n");
			let updated = await rebuild(compiler, [files[0]]);
			expect(reusedPatterns(updated)).toBe(patternCount - 1);
			expect(asset(updated, "copied/value.txt")).toBe(`${winner}\n`);

			write(root, `assets/${winner}.txt`, "last-edited\n");
			updated = await rebuild(compiler, [files[winner]]);
			expect(reusedPatterns(updated)).toBe(patternCount - 1);
			expect(asset(updated, "copied/value.txt")).toBe("last-edited\n");

			const entry = write(
				root,
				"src/index.js",
				"module.exports = 'changed';\n"
			);
			updated = await rebuild(compiler, [entry]);
			expect(reusedPatterns(updated)).toBe(patternCount);
			expect(asset(updated, "copied/value.txt")).toBe("last-edited\n");
		}
	};
}

function globResultOrderCase() {
	const interleavedCount = 32;
	const globFileCount = 64;
	let root;
	let patterns;
	let interleaved;
	let globFiles;

	return {
		description: "should preserve forced glob result order across cache states",
		options(context) {
			patterns = Array.from({ length: interleavedCount }, (_, index) => ({
				from: `assets/interleaved/${index}.txt`,
				to: `copied/interleaved-${index}.txt`,
				toType: "file",
				force: true,
				priority: index % 5
			}));
			patterns.splice(Math.floor(interleavedCount / 2), 0, {
				from: "assets/many/*.txt",
				to: "copied/many.txt",
				toType: "file",
				force: true,
				priority: 2
			});
			const project = createProject(context, "forced-glob-order", patterns);
			root = project.root;
			interleaved = Array.from({ length: interleavedCount }, (_, index) =>
				write(root, `assets/interleaved/${index}.txt`, `${index}\n`)
			);
			globFiles = Array.from({ length: globFileCount }, (_, index) =>
				write(
					root,
					`assets/many/${String(index).padStart(4, "0")}.txt`,
					`${index}\n`
				)
			);
			return project.options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			let updated = await compile(compiler);
			let winner = asset(updated, "copied/many.txt");
			expect(winner).toBeDefined();

			write(root, "assets/interleaved/0.txt", "interleaved-edited\n");
			updated = await rebuild(compiler, [interleaved[0]]);
			expect(reusedPatterns(updated)).toBe(patterns.length - 1);
			expect(asset(updated, "copied/many.txt")).toBe(winner);

			write(root, "assets/many/0000.txt", "glob-edited\n");
			updated = await rebuild(compiler, [globFiles[0]]);
			expect(reusedPatterns(updated)).toBe(patterns.length - 1);
			winner = asset(updated, "copied/many.txt");
			expect(winner).toBeDefined();

			const entry = write(
				root,
				"src/index.js",
				"module.exports = 'changed';\n"
			);
			updated = await rebuild(compiler, [entry]);
			expect(reusedPatterns(updated)).toBe(patterns.length);
			expect(asset(updated, "copied/many.txt")).toBe(winner);
		}
	};
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [patternOrderCase(), globResultOrderCase()];
