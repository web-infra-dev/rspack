import fs from "node:fs";
import path from "node:path";
import { CopyRspackPlugin, Stats } from "@rspack/core";
import {
	compile,
	createProject,
	rebuild,
	useRealOutputFileSystem,
	write
} from "./_copy-plugin-cache.mjs";

const unix = process.platform !== "win32";
const initialMtime = new Date("2020-01-02T03:04:05Z");
const updatedMtime = new Date("2021-02-03T04:05:06Z");
const atime = new Date("2022-03-04T05:06:07Z");
const preserve = { preservePermissions: true, preserveTimestamps: true };
const copySource = "assets/copy.txt";

function writeSource(root, filename) {
	const file = write(root, filename, "copied\n");
	fs.utimesSync(file, atime, initialMtime);
	if (unix) fs.chmodSync(file, 0o751);
	return file;
}

function expectAttributes(file, mtime, mode) {
	const stat = fs.statSync(file);
	expect(stat.mtimeMs).toBe(mtime.getTime());
	if (unix) expect(stat.mode & 0o777).toBe(mode);
	return stat;
}

function preserveNativeAttributes() {
	let source;

	return {
		description:
			"should preserve file attributes and update them without rewriting",
		options(context) {
			const { root, options } = createProject(context, "native", [
				{ from: copySource, ...preserve, info: { version: "same-bytes" } },
				{ from: "assets/legacy.txt", copyPermissions: true },
				{
					from: "assets/legacy.txt",
					to: "disabled.txt",
					copyPermissions: true,
					preservePermissions: false
				}
			]);
			source = writeSource(root, copySource);
			writeSource(root, "assets/legacy.txt");
			return options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			const output = name => path.join(compiler.outputPath, name);
			await compile(compiler);
			const initial = expectAttributes(output("copy.txt"), initialMtime, 0o751);
			const sourceAtime = fs.statSync(source).atimeMs;
			expect(Math.abs(initial.atimeMs - sourceAtime)).toBeLessThan(2);

			const legacy = fs.statSync(output("legacy.txt"));
			expect(legacy.mtimeMs).toBeGreaterThan(initialMtime.getTime());
			if (unix) {
				expect(legacy.mode & 0o777).toBe(0o751);
				expect(fs.statSync(output("disabled.txt")).mode & 0o111).toBe(0);
				fs.chmodSync(source, 0o711);
			}
			fs.utimesSync(source, updatedMtime, updatedMtime);

			const updated = await rebuild(compiler, [source]);
			expectAttributes(output("copy.txt"), updatedMtime, 0o711);
			const { assets } = new Stats(updated).toJson({
				all: false,
				assets: true,
				cachedAssets: true
			});
			expect(assets.find(asset => asset.name === "copy.txt")?.emitted).toBe(false);
		}
	};
}

function reemitChildAsset() {
	return {
		description:
			"should carry pre-transform file attributes through child compilation and re-emission",
		options(context) {
			const { root, options } = createProject(context, "child-copy", []);
			const from = writeSource(root, copySource);
			const copy = new CopyRspackPlugin({
				patterns: [
					{
						from,
						...preserve,
						async transform(content, filename) {
							await fs.promises.utimes(filename, updatedMtime, updatedMtime);
							if (unix) await fs.promises.chmod(filename, 0o711);
							return content;
						}
					}
				]
			});
			options.plugins = [
				compiler => {
					compiler.hooks.make.tapAsync("CopyInChildCompiler", (compilation, callback) => {
						const emitAsset = compilation.emitAsset.bind(compilation);
						compilation.emitAsset = (name, source, info) => {
							if (name !== "copy.txt") {
								return emitAsset(name, source, info);
							}
							emitAsset(name, source, { ...info });
							emitAsset("renamed.txt", source, { ...info });
							emitAsset(name, source, {
								timestamps: { ...info.timestamps, mtime: updatedMtime.getTime() }
							});
							compilation.updateAsset(name, source, {
								timestamps: { atime: initialMtime.getTime() }
							});

							const sharedInfo = { development: true };
							emitAsset(name, source, sharedInfo);
							expect(compilation.getAsset(name).info.timestamps.mtime).toBe(
								updatedMtime.getTime()
							);
							expect(sharedInfo.timestamps).toBeUndefined();
							emitAsset("renamed.txt", source, sharedInfo);
							const inheritedInfo = Object.create({
								mode: 0o600,
								timestamps: { ...info.timestamps, mtime: updatedMtime.getTime() }
							});
							emitAsset("renamed.txt", source, inheritedInfo);
						};
						const child = compilation.createChildCompiler("copy", {}, [copy]);
						child.runAsChild(error => callback(error));
					});
				}
			];
			return options;
		},
		compiler: useRealOutputFileSystem,
		async build(_context, compiler) {
			await compile(compiler);
			const renamed = path.join(compiler.outputPath, "renamed.txt");
			expectAttributes(renamed, initialMtime, 0o751);
			expect(fs.readFileSync(renamed, "utf8")).toBe("copied\n");
			const original = path.join(compiler.outputPath, "copy.txt");
			const stat = expectAttributes(original, updatedMtime, 0o751);
			expect(stat.atimeMs).toBe(initialMtime.getTime());
		}
	};
}

function clearCopiedAttributes() {
	const names = ["emit.txt", "update.txt", "forwarded.txt"];
	return {
		description: "should clear copied file attributes through emitAsset and updateAsset",
		options(context) {
			const { root, options } = createProject(
				context,
				"clear-attributes",
				names.map(to => ({ from: copySource, to, ...preserve }))
			);
			writeSource(root, copySource);
			return options;
		},
		compiler(_context, compiler) {
			compiler.outputFileSystem = { ...fs, chmod: undefined, utimes: undefined };
			compiler.hooks.emit.tap("ClearCopiedAttributes", compilation => {
				const emitted = compilation.getAsset("emit.txt");
				compilation.emitAsset(emitted.name, emitted.source, { mode: undefined });
				const { info } = compilation.getAsset(emitted.name);
				expect(info.mode).toBeUndefined();
				expect(info.timestamps.mtime).toBe(initialMtime.getTime());
				compilation.emitAsset(emitted.name, emitted.source, { timestamps: undefined });
				compilation.emitAsset(emitted.name, emitted.source, { development: true });

				const updated = compilation.getAsset("update.txt");
				compilation.updateAsset(updated.name, updated.source, { timestamps: undefined });
				expect(compilation.getAsset(updated.name).info.timestamps.mtime).toBe(
					initialMtime.getTime()
				);
				compilation.updateAsset(
					updated.name,
					updated.source,
					({ mode, timestamps, ...info }) => info
				);
				expect(Object.hasOwn(compilation.getAsset(updated.name).info, "timestamps")).toBe(false);
				compilation.updateAsset(updated.name, updated.source, () => ({
					mode: undefined,
					timestamps: undefined
				}));
				const forwarded = compilation.getAsset("forwarded.txt");
				compilation.emitAsset(forwarded.name, forwarded.source, {
					...compilation.getAsset(updated.name).info
				});

				for (const name of names) {
					const { info } = compilation.getAsset(name);
					expect(Object.hasOwn(info, "mode")).toBe(true);
					expect(Object.hasOwn(info, "timestamps")).toBe(true);
					expect(info.mode).toBeUndefined();
					expect(info.timestamps).toBeUndefined();
				}
			});
		},
		async build(_context, compiler) {
			await compile(compiler);
			for (const name of names) {
				const output = path.join(compiler.outputPath, name);
				expect(fs.readFileSync(output, "utf8")).toBe("copied\n");
			}
		}
	};
}

const chmodError = "fixture chmod failed";

function rejectChmod(_file, _mode, callback) {
	callback(new Error(chmodError));
}

function permissionFailureCase(option, chmod) {
	const required = option === "preservePermissions";
	const failure = chmod ? "failing chmod" : "missing chmod";

	return {
		description: `should ${required ? "fail" : "warn"} when using ${option} with ${failure}`,
		skip: !unix,
		options(context) {
			const { root, options } = createProject(context, `${option}-${chmod ? "chmod-error" : "no-chmod"}`, [
				{ from: copySource, [option]: true }
			]);
			writeSource(root, copySource);
			return options;
		},
		compiler(_context, compiler) {
			compiler.outputFileSystem = { ...fs, chmod };
			compiler.hooks.emit.tap("ReemitCopiedAsset", compilation => {
				const { source, info } = compilation.getAsset("copy.txt");
				compilation.emitAsset("copy.txt", source, { ...info });
			});
		},
		async build(_context, compiler) {
			if (required) {
				await expect(compile(compiler)).rejects.toThrow(
					chmod ? chmodError : "output file system cannot set file permissions"
				);
				return;
			}
			const compilation = await compile(compiler);
			const { logging } = new Stats(compilation).toJson({
				all: false,
				logging: "warn"
			});
			expect(logging["rspack.Compiler"]?.entries).toContainEqual(
				expect.objectContaining({
					type: "warn",
					message: expect.stringContaining(chmodError)
				})
			);
		}
	};
}

export default [
	preserveNativeAttributes(),
	reemitChildAsset(),
	clearCopiedAttributes(),
	permissionFailureCase("copyPermissions", rejectChmod),
	permissionFailureCase("preservePermissions", rejectChmod),
	permissionFailureCase("preservePermissions", undefined)
];
