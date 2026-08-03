const path = require("node:path");
const {
	asset,
	compile,
	createProject,
	rebuild,
	reusedPatterns,
	useRealOutputFileSystem,
	write
} = require("./_copy-plugin-cache");

function cacheablePatternsCase() {
	let root;
	let transformCalls = 0;
	let destinationCalls = 0;

	return {
		description: "should only cache static patterns",
		options(context) {
			transformCalls = 0;
			destinationCalls = 0;
			const project = createProject(context, "cache-policy", [
				{ from: "assets/static", to: "static" },
				{
					from: "assets/transform",
					to: "transform",
					transform(content) {
						transformCalls += 1;
						return content;
					}
				},
				{
					from: "assets/function",
					to({ absoluteFilename }) {
						destinationCalls += 1;
						return `function/${path.basename(absoluteFilename)}`;
					}
				},
				{ from: "assets/template", to: "template/[name][ext]" },
				{
					from: "assets/nested-template",
					to: "template-path/[path][name][ext]"
				},
				{
					from: "assets/permissions",
					to: "permissions",
					copyPermissions: true
				}
			]);
			root = project.root;
			write(root, "assets/static/one.txt", "static\n");
			write(root, "assets/transform/one.txt", "transform\n");
			write(root, "assets/function/one.txt", "function\n");
			write(root, "assets/template/one.txt", "template\n");
			write(
				root,
				"assets/nested-template/deep/two.txt",
				"nested-template\n"
			);
			write(root, "assets/permissions/one.txt", "permissions\n");
			return project.options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			const initial = await compile(compiler);
			expect(transformCalls).toBe(1);
			expect(destinationCalls).toBe(1);
			expect(asset(initial, "template/one.txt")).toBe("template\n");
			expect(asset(initial, "template-path/deep/two.txt")).toBe(
				"nested-template\n"
			);

			const entry = write(
				root,
				"src/index.js",
				"module.exports = 'changed';\n"
			);
			const updated = await rebuild(compiler, [entry]);

			expect(reusedPatterns(updated)).toBe(1);
			expect(transformCalls).toBe(2);
			expect(destinationCalls).toBe(2);
			expect(asset(updated, "static/one.txt")).toBe("static\n");
			expect(asset(updated, "transform/one.txt")).toBe("transform\n");
			expect(asset(updated, "function/one.txt")).toBe("function\n");
			expect(asset(updated, "template/one.txt")).toBe("template\n");
			expect(asset(updated, "template-path/deep/two.txt")).toBe(
				"nested-template\n"
			);
			expect(asset(updated, "permissions/one.txt")).toBe("permissions\n");
		}
	};
}

function separateRunsCase() {
	let root;
	return {
		description: "should not reuse cached results across separate run calls",
		options(context) {
			const project = createProject(context, "run-twice", [
				{ from: "assets/source", to: "copied" }
			]);
			root = project.root;
			write(root, "assets/source/one.txt", "before\n");
			return project.options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			const initial = await compile(compiler);
			expect(asset(initial, "copied/one.txt")).toBe("before\n");

			write(root, "assets/source/one.txt", "after\n");
			const updated = await compile(compiler);

			expect(asset(updated, "copied/one.txt")).toBe("after\n");
			expect(reusedPatterns(updated)).toBe(0);
		}
	};
}

function emptyRebuildCase() {
	let root;
	return {
		description: "should not reuse cached results for an empty rebuild",
		options(context) {
			const project = createProject(context, "empty-rebuild", [
				{ from: "assets/source", to: "copied" }
			]);
			root = project.root;
			write(root, "assets/source/one.txt", "before\n");
			return project.options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			const initial = await compile(compiler);
			expect(asset(initial, "copied/one.txt")).toBe("before\n");

			write(root, "assets/source/one.txt", "after\n");
			const updated = await rebuild(compiler);

			expect(asset(updated, "copied/one.txt")).toBe("after\n");
			expect(reusedPatterns(updated)).toBe(0);
		}
	};
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [cacheablePatternsCase(), separateRunsCase(), emptyRebuildCase()];
