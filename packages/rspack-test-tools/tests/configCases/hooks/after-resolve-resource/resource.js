import a from "./a";
import b from "./b";
import fs from "fs";

it("should modify resource by after resolve hook", () => {
  expect(a).toBe("a")
  expect(b).toBe("c")
  const ext = ".js";
  expect(fs.readFileSync(__filename, "utf-8")).toContain("./b" + ext);
});
