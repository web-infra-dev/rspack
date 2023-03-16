import fs from "fs";
require("./img.png");

it("should create the same asset as input", () => {
	const png = fs.readdirSync(__dirname).find(file => file.endsWith(".png"));
	expect(fs.readFileSync(__dirname + "/" + png)).toEqual(
		fs.readFileSync(__dirname + "/../" + "img.png")
	);
});
