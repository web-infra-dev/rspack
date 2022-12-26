import fs from "node:fs";

export async function editFile(
	filename: string,
	replacer: (str: string) => string
): Promise<void> {
	const content = fs.readFileSync(filename, "utf-8");
	const modified = replacer(content);
	fs.writeFileSync(filename, modified);
	return new Promise(resolve => {
		setTimeout(() => {
			resolve(undefined);
		}, 1000);
	});
}
