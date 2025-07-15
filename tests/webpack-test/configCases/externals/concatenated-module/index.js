import fs1 from "fs";
import fsPromises1 from "fs-promises";
import fs2 from "module-fs";
import fsPromises2 from "module-fs-promises";
import url2 from "module-import-url";
import path2 from "module-path";
import path1 from "path";
import url1 from "url";

it("should be possible to import multiple module externals", () => {
	expect(fs2).toBe(fs1);
	expect(path2).toBe(path1);
	expect(fsPromises2).toBe(fsPromises1);
	expect(url1).toBe(url2);
});
