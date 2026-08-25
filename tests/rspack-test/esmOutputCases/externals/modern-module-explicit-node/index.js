import { resolve } from "path";

const fs = require("fs");

export const readFile = fs.readFile;
export const pathResolve = resolve;

export async function loadPlatform() {
	const os = await import("os");
	return os.platform;
}
