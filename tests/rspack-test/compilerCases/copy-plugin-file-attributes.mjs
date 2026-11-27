import fs from "node:fs";
import path from "node:path";
import {
	compile,
	createProject,
	rebuild,
	useRealOutputFileSystem,
	write
} from "./_copy-plugin-cache.mjs";

const initialTime = new Date("2020-01-02T03:04:05Z");
const updatedTime = new Date("2021-02-03T04:05:06Z");
const copySource = "assets/copy.txt";
const supportsUnixMode = process.platform !== "win32" && !process.env.WASM;

export default [{
	description: "should preserve fresh copy attributes and content across builds",
	options(context) {
		const { root, options } = createProject(context, "file-attributes", [{
			from: copySource,
			preservePermissions: true,
			preserveTimestamps: true,
			noErrorOnMissing: true,
			info: { version: "stable" }
		}]);
		const source = write(root, copySource, "copied\n");
		fs.utimesSync(source, initialTime, initialTime);
		if (supportsUnixMode) fs.chmodSync(source, 0o751);
		return options;
	},
	compiler: useRealOutputFileSystem,
	async build(_context, compiler) {
		const source = path.join(compiler.context, copySource);
		const output = path.join(compiler.outputPath, "copy.txt");

		await compile(compiler);
		const initial = fs.statSync(output);
		expect(initial.mtimeMs).toBe(initialTime.getTime());
		if (supportsUnixMode) expect(initial.mode & 0o777).toBe(0o751);

		fs.utimesSync(source, updatedTime, updatedTime);
		if (supportsUnixMode) fs.chmodSync(source, 0o711);
		await rebuild(compiler);
		const updated = fs.statSync(output);
		expect(updated.mtimeMs).toBe(updatedTime.getTime());
		if (supportsUnixMode) expect(updated.mode & 0o777).toBe(0o711);

		fs.writeFileSync(source, "edited\n");
		fs.utimesSync(source, initialTime, initialTime);
		await compile(compiler);
		expect(fs.readFileSync(output, "utf8")).toBe("edited\n");

		fs.unlinkSync(source);
		const compilation = await compile(compiler);
		expect(compilation.getAsset("copy.txt")).toBeUndefined();
	}
}];
