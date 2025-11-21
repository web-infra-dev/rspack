it("should generate correct relative sourceMappingURL when fileContext is passed", function () {
    var fs = require("fs"),
        path = require("path");
    var source = fs.readFileSync(path.join(__dirname, "main.js"), "utf-8");
    expect(source).toMatch("//# sourceMappingURL=http://localhost:50505/sourcemaps/../main.js.map");
});
