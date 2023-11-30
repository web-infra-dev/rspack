import { Button } from "aaaaa";
import fs from "fs";

it("should generate css successfully", () => {
	const dir = fs.readdirSync(__dirname);
	expect(dir).toStrictEqual(["main.css", "main.js"]);
	expect(Button).toBe("button");
});
