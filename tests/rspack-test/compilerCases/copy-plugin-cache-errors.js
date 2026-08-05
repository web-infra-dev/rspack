const path = require("node:path");
const {
	asset,
	compile,
	createProject,
	rebuild,
	rebuildAllowErrors,
	remove,
	reusedPatterns,
	useRealOutputFileSystem,
	write
} = require("./_copy-plugin-cache");

function diagnosticRecoveryCase() {
	let root;
	return {
		description: "should evict an errored pattern and recover when it returns",
		options(context) {
			const project = createProject(context, "diagnostic-recovery", [
				{
					from: "assets/source/*.txt",
					to: "copied",
					toType: "dir"
				}
			]);
			root = project.root;
			write(root, "assets/source/one.txt", "before\n");
			return project.options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			const source = path.join(root, "assets/source/one.txt");
			const initial = await compile(compiler);
			expect(asset(initial, "copied/assets/source/one.txt")).toBe("before\n");

			remove(root, "assets/source/one.txt");
			let failed = await rebuildAllowErrors(compiler, [], [source]);
			expect(failed.errors.length).toBeGreaterThan(0);
			expect(
				failed.errors.filter(error =>
					String(error.message ?? error).includes("unable to locate")
				)
			).toHaveLength(1);
			expect(reusedPatterns(failed)).toBe(0);
			expect(
				asset(failed, "copied/assets/source/one.txt")
			).toBeUndefined();

			const entry = write(
				root,
				"src/index.js",
				"module.exports = 'changed';\n"
			);
			failed = await rebuildAllowErrors(compiler, [entry]);
			expect(failed.errors.length).toBeGreaterThan(0);
			expect(reusedPatterns(failed)).toBe(0);
			expect(
				asset(failed, "copied/assets/source/one.txt")
			).toBeUndefined();

			write(root, "assets/source/one.txt", "after\n");
			const recovered = await rebuild(compiler, [source]);
			expect(asset(recovered, "copied/assets/source/one.txt")).toBe(
				"after\n"
			);
			expect(reusedPatterns(recovered)).toBe(0);
		}
	};
}

function siblingDiagnosticCase() {
	let root;
	return {
		description: "should keep a successful sibling cached after another errors",
		options(context) {
			const project = createProject(context, "sibling-diagnostic", [
				{
					from: "assets/missing/*.txt",
					to: "copied",
					toType: "dir"
				},
				{
					from: "assets/stable.txt",
					to: "copied/stable.txt",
					toType: "file"
				}
			]);
			root = project.root;
			write(root, "assets/missing/one.txt", "one\n");
			write(root, "assets/stable.txt", "before\n");
			return project.options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			const missing = path.join(root, "assets/missing/one.txt");
			const stable = path.join(root, "assets/stable.txt");
			await compile(compiler);
			remove(root, "assets/missing/one.txt");
			write(root, "assets/stable.txt", "after\n");
			let failed = await rebuildAllowErrors(compiler, [stable], [missing]);

			expect(failed.errors).toHaveLength(1);
			expect(asset(failed, "copied/stable.txt")).toBe("after\n");
			expect(reusedPatterns(failed)).toBe(0);

			const entry = write(
				root,
				"src/index.js",
				"module.exports = 'changed';\n"
			);
			failed = await rebuildAllowErrors(compiler, [entry]);

			expect(failed.errors).toHaveLength(1);
			expect(asset(failed, "copied/stable.txt")).toBe("after\n");
			expect(reusedPatterns(failed)).toBe(1);
		}
	};
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [diagnosticRecoveryCase(), siblingDiagnosticCase()];
