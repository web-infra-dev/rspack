import a from "./a";
import b from "./b"; // b.js will be transform to c.js
import c from "./c";
import fs from "fs";

it("should remove duplicate request modules generate by after resolve hook", () => {
	expect(a).toBe("a");
	expect(b).toBe("c");
	expect(c).toBe("c");
	const ext = ".js";
	expect(fs.readFileSync(__filename, "utf-8")).not.toContain("./b" + ext);
	expect(fs.readFileSync(__filename, "utf-8")).toContain("./c" + ext);
	expect(
		fs.readFileSync(__filename, "utf-8").split("./c" + ext).length - 1
	).toBe(2);
});
