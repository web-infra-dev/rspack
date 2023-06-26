import { dirname, filename } from "./child/child";

it("dirname mock", function () {
	expect(__dirname).toBe("");
	expect(dirname).toBe("child");
	expect(__filename).toBe("index.js");
	console.log(filename);
	expect(filename.replace(/\\/g, "/")).toBe("child/child.js");
});
