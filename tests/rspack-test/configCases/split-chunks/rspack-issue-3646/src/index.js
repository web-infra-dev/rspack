import fs from "fs";
import path from "path";
import "./shared";
export default "index.js";

it("issue-3646", () => {
	expect(fs.existsSync(path.resolve(__dirname, "./another.js"))).toBe(true);
	expect(
		fs.readdirSync(path.resolve(__dirname)).some(p => p.includes("lodash"))
	).toBe(true);
});
