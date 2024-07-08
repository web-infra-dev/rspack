import { Button } from "aaaaa";
import fs from "fs";

it("should generate css successfully", () => {
	const dir = fs.readdirSync(__dirname);
	expect(dir.includes("bundle0.js")).toBeTruthy();
	expect(dir.includes("bundle0.css")).toBeTruthy();
	expect(Button).toBe("button");
});
