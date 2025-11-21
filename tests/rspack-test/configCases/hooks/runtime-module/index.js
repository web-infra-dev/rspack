import { a } from "./chunk";
import fs from "fs";
import path from "path";

it("should modify runtime module source in main", () => {
	expect(__webpack_require__.test).toBeTruthy();
	expect(a).toBeTruthy();
});

it("should not modify runtime module source in chunk", () => {
	expect(
		fs.readFileSync(path.join(__dirname, "chunk.js"), "utf-8")
	).not.toContain("__webpack_require__.test = true");
});
