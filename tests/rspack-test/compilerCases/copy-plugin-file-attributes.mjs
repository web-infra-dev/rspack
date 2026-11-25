import fs from "node:fs";
import path from "node:path";
import { Compilation, CopyRspackPlugin, Stats } from "@rspack/core";
import {
	compile,
	createProject,
	rebuild,
	useRealOutputFileSystem,
	write
} from "./_copy-plugin-cache.mjs";

const initialMtime = new Date("2020-01-02T03:04:05Z");
const updatedMtime = new Date("2021-02-03T04:05:06Z");
const atime = new Date("2022-03-04T05:06:07Z");
const unix = process.platform !== "win32";

function preserveFileAttributes(useInputFileSystem) {
	let root;
	let source;
	let writes;

	return {
		description: `should preserve file attributes from the ${useInputFileSystem ? "JS" : "native"} input file system`,
		options(context) {
			const project = createProject(
				context,
				useInputFileSystem ? "js-input" : "native-input",
				[
					{
						from: "assets/preserved.txt",
						preservePermissions: true,
						preserveTimestamps: true,
						info: { version: "same-bytes" }
					},
					{ from: "assets/legacy.txt", copyPermissions: true },
					{
						from: "assets/disabled.txt",
						copyPermissions: true,
						preservePermissions: false
					}
				]
			);
			root = project.root;
			source = write(root, "assets/preserved.txt", "preserved\n");
			fs.utimesSync(source, atime, initialMtime);
			for (const name of ["legacy", "disabled"]) {
				const file = write(root, `assets/${name}.txt`, `${name}\n`);
				fs.utimesSync(file, atime, initialMtime);
				if (unix) fs.chmodSync(file, 0o751);
			}
			if (unix) fs.chmodSync(source, 0o751);
			if (useInputFileSystem) {
				project.options.experiments = {
					useInputFileSystem: [/[\\/]assets[\\/]/]
				};
			}
			return project.options;
		},
		compiler(_context, compiler) {
			writes = 0;
			compiler.inputFileSystem = fs;
			compiler.outputFileSystem = {
				...fs,
				writeFile(file, content, callback) {
					if (path.basename(file) === "preserved.txt") writes++;
					fs.writeFile(file, content, callback);
				}
			};
		},
		async build(_context, compiler) {
			await compile(compiler);
			const target = path.join(root, "dist", "preserved.txt");
			const initial = fs.statSync(target);
			expect(initial.mtimeMs).toBe(initialMtime.getTime());
			expect(Math.abs(initial.atimeMs - fs.statSync(source).atimeMs)).toBeLessThan(2);
			expect(writes).toBe(1);

			const legacy = fs.statSync(path.join(root, "dist", "legacy.txt"));
			const disabled = fs.statSync(path.join(root, "dist", "disabled.txt"));
			expect(legacy.mtimeMs).toBeGreaterThan(initialMtime.getTime());
			if (unix) {
				expect(initial.mode & 0o777).toBe(0o751);
				expect(legacy.mode & 0o777).toBe(0o751);
				expect(disabled.mode & 0o111).toBe(0);
				fs.chmodSync(source, 0o711);
			}
			fs.utimesSync(source, atime, updatedMtime);

			await rebuild(compiler, [source]);
			const updated = fs.statSync(target);
			expect(updated.mtimeMs).toBe(updatedMtime.getTime());
			if (unix) expect(updated.mode & 0o777).toBe(0o711);
			expect(writes).toBe(1);
		}
	};
}

function preserveAttributesBeforeTransform() {
	let source;
	let target;

	return {
		description: "should read file attributes before running an async transform",
		options(context) {
			let updateSource = true;
			const { root, options } = createProject(context, "transform", [
				{
					from: "assets/transformed.txt",
					preservePermissions: true,
					preserveTimestamps: true,
					async transform(content) {
						if (updateSource) {
							updateSource = false;
							await fs.promises.writeFile(source, "after!\n");
							await fs.promises.utimes(source, atime, updatedMtime);
							if (unix) await fs.promises.chmod(source, 0o711);
						}
						return content;
					}
				}
			]);
			source = write(root, "assets/transformed.txt", "before\n");
			fs.utimesSync(source, atime, initialMtime);
			if (unix) fs.chmodSync(source, 0o751);
			target = path.join(root, "dist", "transformed.txt");
			return options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			await compile(compiler);
			const initial = fs.statSync(target);
			expect(fs.readFileSync(target, "utf8")).toBe("before\n");
			expect(initial.mtimeMs).toBe(initialMtime.getTime());
			if (unix) expect(initial.mode & 0o777).toBe(0o751);

			await rebuild(compiler, [source]);
			const updated = fs.statSync(target);
			expect(fs.readFileSync(target, "utf8")).toBe("after!\n");
			expect(updated.mtimeMs).toBe(updatedMtime.getTime());
			if (unix) expect(updated.mode & 0o777).toBe(0o711);
		}
	};
}

const chmodError = "fixture chmod failed";

function rejectChmod(_file, _mode, callback) {
	callback(new Error(chmodError));
}

function afterCopy(processAssets) {
	return {
		apply(compiler) {
			compiler.hooks.thisCompilation.tap("ReemitCopiedAsset", compilation => {
				compilation.hooks.processAssets.tap(
					{
						name: "ReemitCopiedAsset",
						stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE
					},
					() => processAssets(compilation)
				);
			});
		}
	};
}

function preserveReemittedFileAttributes() {
	let root;

	return {
		description: "should preserve file attributes when re-emitting a copied asset",
		options(context) {
			const project = createProject(context, "reemit", [
				{
					from: "assets/copy.txt",
					preservePermissions: true,
					preserveTimestamps: true
				}
			]);
			root = project.root;
			const sourceFile = write(root, "assets/copy.txt", "copied\n");
			fs.utimesSync(sourceFile, atime, initialMtime);
			if (unix) fs.chmodSync(sourceFile, 0o751);
			project.options.plugins.push(
				afterCopy(compilation => {
					const { source, info } = compilation.getAsset("copy.txt");
					expect(info.timestamps.mtime).toBe(initialMtime.getTime());
					if (unix) expect(info.mode).toBe(0o751);
					compilation.emitAsset("copy.txt", source, { ...info });
					compilation.emitAsset("copy.txt", source, {
						timestamps: { ...info.timestamps, mtime: updatedMtime.getTime() }
					});
					compilation.emitAsset("renamed.txt", source, { ...info });
				})
			);
			return project.options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			await compile(compiler);
			for (const [name, mtime] of [
				["copy.txt", updatedMtime],
				["renamed.txt", initialMtime]
			]) {
				const target = path.join(root, "dist", name);
				const stat = fs.statSync(target);
				expect(stat.mtimeMs).toBe(mtime.getTime());
				if (unix) expect(stat.mode & 0o777).toBe(0o751);
				expect(fs.readFileSync(target, "utf8")).toBe("copied\n");
			}
		}
	};
}

function preserveChildCompilerFileAttributes() {
	let root;

	return {
		description:
			"should preserve child copy attributes through the parent emitAsset method",
		options(context) {
			const project = createProject(context, "child-compiler", []);
			root = project.root;
			const source = write(root, "assets/child.txt", "copied by a child\n");
			fs.utimesSync(source, atime, initialMtime);
			if (unix) fs.chmodSync(source, 0o751);
			project.options.plugins = [
				{
					apply(compiler) {
						compiler.hooks.make.tapAsync(
							"CopyInChildCompiler",
							(compilation, callback) => {
								const emitAsset = compilation.emitAsset.bind(compilation);
								compilation.emitAsset = (name, source, info) => {
									emitAsset(name === "child.txt" ? "routed.txt" : name, source, info);
								};
								const child = compilation.createChildCompiler("copy", {}, [
									new CopyRspackPlugin({
										patterns: [
											{
												from: source,
												preservePermissions: true,
												preserveTimestamps: true
											}
										]
									})
								]);
								child.runAsChild(error => callback(error));
							}
						);
					}
				}
			];
			return project.options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			await compile(compiler);
			const target = path.join(root, "dist", "routed.txt");
			const stat = fs.statSync(target);
			expect(stat.mtimeMs).toBe(initialMtime.getTime());
			if (unix) expect(stat.mode & 0o777).toBe(0o751);
			expect(fs.readFileSync(target, "utf8")).toBe("copied by a child\n");
		}
	};
}

function permissionFailureCase(option, chmod) {
	const required = option === "preservePermissions";
	const failure = chmod ? "failing chmod" : "missing chmod";
	let target;

	return {
		description: `should ${required ? "fail" : "continue"} when using ${option} with ${failure}`,
		skip: !unix,
		options(context) {
			const { root, options } = createProject(
				context,
				`${option}-${chmod ? "chmod-error" : "no-chmod"}`,
				[{ from: "assets/copy.txt", [option]: true }]
			);
			const source = write(root, "assets/copy.txt", "copied\n");
			fs.chmodSync(source, 0o751);
			target = path.join(root, "dist", "copy.txt");
			options.plugins.push(
				afterCopy(compilation => {
					const { source, info } = compilation.getAsset("copy.txt");
					compilation.emitAsset("copy.txt", source, { ...info });
				})
			);
			return options;
		},
		compiler(_context, compiler) {
			compiler.outputFileSystem = { ...fs, chmod };
		},
		async build(_context, compiler) {
			if (required) {
				await expect(compile(compiler)).rejects.toThrow(
					chmod ? chmodError : "output file system cannot set file permissions"
				);
				return;
			}
			const compilation = await compile(compiler);
			expect(fs.readFileSync(target, "utf8")).toBe("copied\n");
			const { logging } = new Stats(compilation).toJson({
				all: false,
				logging: "warn"
			});
			const entries = logging?.["rspack.Compiler"]?.entries ?? [];
			const warnings = entries
				.filter(
					entry =>
						entry.type === "warn" &&
						entry.message.includes("Failed to set permissions")
				)
				.map(entry => entry.message);
			expect(warnings).toEqual(
				chmod ? [expect.stringContaining(chmodError)] : []
			);
		}
	};
}

export default [
	preserveFileAttributes(false),
	preserveFileAttributes(true),
	preserveAttributesBeforeTransform(),
	preserveReemittedFileAttributes(),
	preserveChildCompilerFileAttributes(),
	...["copyPermissions", "preservePermissions"].flatMap(option => [
		permissionFailureCase(option, undefined),
		permissionFailureCase(option, rejectChmod)
	])
];
