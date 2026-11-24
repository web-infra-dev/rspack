import fs from "node:fs";
import path from "node:path";
import { compile, createProject, rebuild, write } from "./_copy-plugin-cache.mjs";

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

export default [preserveFileAttributes(false), preserveFileAttributes(true)];
