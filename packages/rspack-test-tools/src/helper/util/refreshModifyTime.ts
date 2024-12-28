import { readFile, writeFile } from "fs-extra";

export async function refreshModifyTime(file: string) {
	const data = await readFile(file);
	await writeFile(file, data);
}
