import fs from "fs";
import path from "path";
import { effects, live, usedFeature } from "./module";
import "./default-unused";
import usedDefaultFeature from "./default-used";

it("should omit dynamic imports in unused object property functions", async () => {
  expect(live).toBe("live");
  expect(effects).toEqual([
    "unused arrow",
    "unused function",
    "unused nested loader",
    "used loader",
    "eager import",
    "unused default",
    "used default",
  ]);

  const used = await usedFeature.loader();
  expect(used.value).toBe("used dynamic import");
  const usedDefault = await usedDefaultFeature.loader();
  expect(usedDefault.value).toBe("used default dynamic import");

  expect(fs.existsSync(path.join(__dirname, "unused.js"))).toBe(false);
  expect(fs.existsSync(path.join(__dirname, "unused-nested.js"))).toBe(false);
  expect(fs.existsSync(path.join(__dirname, "default-unused.js"))).toBe(false);
  expect(fs.existsSync(path.join(__dirname, "used.js"))).toBe(true);
  expect(fs.existsSync(path.join(__dirname, "default-used.js"))).toBe(true);
  expect(fs.existsSync(path.join(__dirname, "eager.js"))).toBe(true);
});
