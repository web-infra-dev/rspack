import fs from "fs";
import path from "path";
import { disposalImport } from "./module";

it("should keep dynamic imports used by implicit resource disposal", async () => {
  const disposed = await disposalImport;
  expect(disposed.value).toBe("resource disposed");
  expect(fs.existsSync(path.join(__dirname, "dispose.js"))).toBe(true);
});
