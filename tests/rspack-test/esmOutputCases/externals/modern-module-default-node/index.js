import { resolve } from "path";

const fs = require("fs");

export const readFile = fs.readFile;
export const pathResolve = resolve;

export async function loadPlatform() {
	const os = await import("os");
	return os.platform;
}

it("should choose external rendering from dependency type for modern-module", async () => {
	expect(readFile).toBe(fs.readFile);
	expect(pathResolve).toBe(resolve);
	expect(await loadPlatform()).toBeDefined();
});
