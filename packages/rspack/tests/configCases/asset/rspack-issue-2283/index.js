import fs from "fs";

it("should create the same asset as input", () => {
	expect(fs.readFileSync(__dirname + "/" + require("./img.png"), "utf-8")).toBe(
		""
	);
});
