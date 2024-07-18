import fs from "fs";
import path from "path";
import "./a.txt";

it("should generate asset using the filename function", () => {
    expect(fs.existsSync(path.join(__STATS__.outputPath, "text/a.txt"))).toBe(true);
});
