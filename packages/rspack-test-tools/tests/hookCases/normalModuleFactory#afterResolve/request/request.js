import a from "./a";
import b from "./b";
import fs from "fs";

it("should modify request by after resolve hook", () => {
	expect(a).toBe("a");
	expect(b).toBe("b");
	const ext = ".js";
	expect(fs.readFileSync(__filename, "utf-8")).toContain("./c" + ext);
});
