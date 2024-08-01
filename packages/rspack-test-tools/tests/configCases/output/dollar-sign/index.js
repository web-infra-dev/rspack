import './img$s/a$b.png';
import fs from "fs";
import path from "path";
it("should replace filename with dollar sign", async function () {
  expect(fs.existsSync(path.resolve(__dirname, './img$s/a$b.png'))).toBe(true);
});
