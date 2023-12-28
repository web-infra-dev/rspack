import fs from "node:fs";
import url from "node:url";

export default function () {
	console.info("hello world");
}

export const add = (a, b) => {
	return a + b;
};

it("should run", function () {});

it("should export module library", function () {
	const source = fs.readFileSync(url.fileURLToPath(import.meta.url), "utf-8");
	expect(source).toContain(`hello world`);
});
