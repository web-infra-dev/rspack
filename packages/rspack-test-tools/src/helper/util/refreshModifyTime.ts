import { readFile, stat, writeFile } from "fs-extra";

export async function refreshModifyTime(file: string) {
	const before = await stat(file);
	const data = await readFile(file);
	await writeFile(file, data);
	const after = await stat(file);
	console.log(before.mtime, after.mtime);
}
