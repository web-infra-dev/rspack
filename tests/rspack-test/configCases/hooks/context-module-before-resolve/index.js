it("should compile", async () => {
    try {
        ["fr.js"].map(file => {
            require("./locale/" + file);
        });
    } catch (e) {
        expect(e.message).toContain("Cannot find module './fr.js'")
    }
    ["zh.js"].map(file => {
        expect(require("./locale/" + file).default).toBe("你好");
    });
    ["en.js"].map(file => {
        expect(require("./locale/" + file).default).toBe("hello");
    });
});
