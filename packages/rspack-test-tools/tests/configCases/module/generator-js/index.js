import { lib } from "./lib";
import fs from 'node:fs';
import path from 'node:path';

it("compiled success and generator should ignored", () => {
	expect(lib).toEqual(42);
	const content = fs.readFileSync(path.resolve(__dirname, "main.js"), "utf-8");
	expect(content).toMatch("42");
});
