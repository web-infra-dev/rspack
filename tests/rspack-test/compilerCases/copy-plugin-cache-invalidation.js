const fs = require("node:fs");
const path = require("node:path");
const {
	asset,
	compile,
	createProject,
	rebuild,
	remove,
	reusedPatterns,
	useRealOutputFileSystem,
	write
} = require("./_copy-plugin-cache");

function globSiblingCase() {
	let root;
	return {
		description: "should track a stable glob base when a new sibling matches",
		options(context) {
			const project = createProject(context, "glob-sibling", [
				{ from: "assets/*/*.txt", to: "copied", toType: "dir" }
			]);
			root = project.root;
			write(root, "assets/a/one.txt", "one\n");
			fs.mkdirSync(path.join(root, "assets/b"), {
				recursive: true
			});
			return project.options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			const initial = await compile(compiler);
			expect([...initial.contextDependencies]).toContain(
				path.join(root, "assets")
			);
			expect(asset(initial, "copied/assets/a/one.txt")).toBe("one\n");

			const sibling = write(root, "assets/b/two.txt", "two\n");
			let updated = await rebuild(compiler, [sibling]);
			expect(reusedPatterns(updated)).toBe(0);
			expect(asset(updated, "copied/assets/b/two.txt")).toBe("two\n");

			const entry = write(
				root,
				"src/index.js",
				"module.exports = 'changed';\n"
			);
			updated = await rebuild(compiler, [entry]);
			expect(reusedPatterns(updated)).toBe(1);
			expect([...updated.contextDependencies]).toContain(
				path.join(root, "assets")
			);
			expect([...updated.fileDependencies]).toEqual(
				expect.arrayContaining([
					path.join(root, "assets/a/one.txt"),
					sibling
				])
			);
			expect(asset(updated, "copied/assets/a/one.txt")).toBe("one\n");
			expect(asset(updated, "copied/assets/b/two.txt")).toBe("two\n");
		}
	};
}

function ancestorCase(kind) {
	let root;
	const from =
		kind === "file" ? "assets/nested/one.txt" : "assets/nested";
	const emitted =
		kind === "file" ? "copied-file/one.txt" : "copied-dir/one.txt";

	return {
		description: `should invalidate a ${kind} pattern for an ancestor event`,
		options(context) {
			const project = createProject(context, `ancestor-${kind}`, [
				{
					from,
					to: kind === "file" ? "copied-file" : "copied-dir",
					toType: "dir"
				}
			]);
			root = project.root;
			write(root, "assets/nested/one.txt", "before\n");
			return project.options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			const initial = await compile(compiler);
			expect(asset(initial, emitted)).toBe("before\n");

			write(root, "assets/nested/one.txt", "after\n");
			const updated = await rebuild(compiler, [path.join(root, "assets")]);

			expect(asset(updated, emitted)).toBe("after\n");
			expect(reusedPatterns(updated)).toBe(0);
		}
	};
}

function changedAndRemovedChildrenCase() {
	let root;
	return {
		description: "should invalidate modified, removed, and newly matching children",
		options(context) {
			const project = createProject(context, "glob-add-remove", [
				{
					from: "assets/glob/**/*.txt",
					to: "copied",
					toType: "dir",
					noErrorOnMissing: true,
					globOptions: { ignore: ["**/skip-*.txt"] }
				}
			]);
			root = project.root;
			write(root, "assets/glob/nested/one.txt", "one\n");
			return project.options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			const one = path.join(root, "assets/glob/nested/one.txt");
			const initial = await compile(compiler);
			expect(asset(initial, "copied/assets/glob/nested/one.txt")).toBe(
				"one\n"
			);

			const two = write(root, "assets/glob/nested/two.txt", "two\n");
			let updated = await rebuild(compiler, [two]);
			expect(asset(updated, "copied/assets/glob/nested/two.txt")).toBe(
				"two\n"
			);

			write(root, "assets/glob/nested/two.txt", "two-edited\n");
			updated = await rebuild(compiler, [two]);
			expect(asset(updated, "copied/assets/glob/nested/two.txt")).toBe(
				"two-edited\n"
			);

			remove(root, "assets/glob/nested/one.txt");
			remove(root, "assets/glob/nested/two.txt");
			updated = await rebuild(compiler, [], [one, two]);
			expect(
				asset(updated, "copied/assets/glob/nested/one.txt")
			).toBeUndefined();
			expect(
				asset(updated, "copied/assets/glob/nested/two.txt")
			).toBeUndefined();

			const three = write(root, "assets/glob/other/three.txt", "three\n");
			const ignored = write(
				root,
				"assets/glob/other/skip-three.txt",
				"ignored\n"
			);
			updated = await rebuild(compiler, [three, ignored]);
			expect(asset(updated, "copied/assets/glob/other/three.txt")).toBe(
				"three\n"
			);
			expect(
				asset(updated, "copied/assets/glob/other/skip-three.txt")
			).toBeUndefined();
		}
	};
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
	globSiblingCase(),
	ancestorCase("directory"),
	ancestorCase("file"),
	changedAndRemovedChildrenCase()
];
