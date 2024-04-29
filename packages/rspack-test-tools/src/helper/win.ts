import path from "path";

export function escapeSep(str: string) {
	return str.split(path.win32.sep).join(path.posix.sep);
}

export function escapeEOL(str: string) {
	return str
		.split("\r\n")
		.map(i => i.trim())
		.join("\n")
		.trim();
}
