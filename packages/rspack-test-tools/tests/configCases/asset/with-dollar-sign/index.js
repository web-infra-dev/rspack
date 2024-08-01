import "./img$1.png";
import "./img$2.png";
import fs from "fs";
import path from "path";

it("should compile", () => {
	expect(fs.existsSync(path.resolve(__dirname, 'img$1.png'))).toBeTruthy();
	expect(fs.existsSync(path.resolve(__dirname, 'img$2.png'))).toBeTruthy();
});
