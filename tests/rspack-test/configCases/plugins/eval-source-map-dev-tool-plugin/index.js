it("should generate the correct sourceRoot in SourceMap", () => {
    const fs = require("fs");
    const source = fs.readFileSync(__filename, "utf-8");
    const base64 =
        /sourceMappingURL\s*=\s*data:application\/json;charset=utf-8;base64,(.*)\"\);/.exec(
            source
        )[1];
    const map = JSON.parse(Buffer.from(base64, "base64").toString("utf-8"));
    expect(map.sources[0]).toMatch(/webpack:\/\/\/\.\/index.js?[a-zA-Z0-9]+/);
    expect(map.sourceRoot).toMatch(path.resolve(CONTEXT, "./folder") + "/");
});

