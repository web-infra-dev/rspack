import fs from "fs";
import path from "path";

it("should successfully emit foo.txt", () => {
    const txt = fs.readFileSync(path.join(__dirname, "foo.txt"), "utf-8");
    expect(txt).toBe("foo");
});
