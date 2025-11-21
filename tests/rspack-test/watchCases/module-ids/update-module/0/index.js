import fs from "fs/promises";
import { value } from "./a";

it("should not change the module id for the updated module", async () => {
  const content = await fs.readFile(__filename, 'utf-8');
  expect(/"\.\/a\.js": \(.*\) {/.test(content)).toBe(true);
  expect(value).toBe(WATCH_STEP);
})

it("should have correct log when incremental enabled", async () => {
  const fs = require("fs/promises");
  const path = require("path");
  const statsString = await fs.readFile(path.resolve(__dirname, `stats.${WATCH_STEP}.txt`), 'utf-8');
  const incrementalLog = /LOG from rspack\.incremental\.moduleIds[\s\S]*?LOG/.exec(statsString);
  if (incrementalLog) {
    const content = incrementalLog[0];
    switch (WATCH_STEP) {
      case "0":
        expect(content.includes("4 modules are affected, 4 in total")).toBe(true);
        expect(content.includes("4 modules are updated by set_module_id, with 0 unnamed modules")).toBe(true);
        break;
      case "1":
        expect(content.includes("1 modules are affected, 4 in total")).toBe(true);
        expect(content.includes("0 modules are updated by set_module_id, with 0 unnamed modules")).toBe(true);
        break;
    }
  }
});
