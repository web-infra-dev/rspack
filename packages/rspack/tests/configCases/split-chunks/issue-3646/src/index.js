import fs from "fs";
import path from "path";
import "./shared";
export default "index.js";

it("issue-3646", () => {
	expect(fs.existsSync(path.resolve(__dirname, "./defaultVendors.js"))).toBe(
		true
	);
	expect(fs.existsSync(path.resolve(__dirname, "./default.js"))).toBe(false);
});
