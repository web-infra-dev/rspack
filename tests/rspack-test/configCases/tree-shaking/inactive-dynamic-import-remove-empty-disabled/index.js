import fs from "fs";
import path from "path";
import { effects, live } from "./module";

it("should retain empty chunk graph metadata when removal is disabled", () => {
  expect(effects).toEqual(["module evaluated"]);
  expect(live).toBe("live");
  expect(fs.existsSync(path.join(__dirname, "dead.js"))).toBe(false);
});
