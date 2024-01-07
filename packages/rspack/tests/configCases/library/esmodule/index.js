import fs from "node:fs";
import { URL } from "node:url";

export default function () {
	console.info("hello world");
}

export const add = (a, b) => {
	return a + b;
};

it("should run", function () {});

it("should export module library", function () {
	const filename = import.meta.url;
	const source = fs.readFileSync(new URL("./dist/main.js", filename), "utf-8");
	// Expected: export { add as add, default as default };
	expect(source).toMatch(/export\s?{\s?\w+ as add,\s?\w+ as default\s?};?\s*$/);
});
