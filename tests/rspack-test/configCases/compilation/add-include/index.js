import fs from "fs";
import path from "path";

it("should ensure proper module requirement functionality using the manifest's mapping", () => {
    const manifest = JSON.parse(fs.readFileSync(path.join(__dirname, "manifest.json"), "utf-8"));
    expect(__webpack_require__(manifest["foo"]).foo).toBe("foo");
    expect(__webpack_require__(manifest["bar"]).bar).toBe("bar");
});
