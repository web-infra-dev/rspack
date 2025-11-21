import fs from "fs";
import path from "path";
import name from "./a.txt";

it("should use correct generator options", async () => {
	expect(fs.existsSync(path.join(__dirname, "assets/a.txt"))).toBeTruthy();
	expect(name).toEqual("https://cdn/assets/a.txt");
});
