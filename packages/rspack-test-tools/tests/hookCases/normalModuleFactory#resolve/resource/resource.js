import fs from "fs";
import a from "./a";
import b from "./b";

it("should modify resource by resolve hook", () => {
	expect(a).toBe("a");
	expect(b).toBe("c");
	const ext = ".js";
	expect(fs.readFileSync(__filename, "utf-8")).toContain("resource/b" + ext);
});
