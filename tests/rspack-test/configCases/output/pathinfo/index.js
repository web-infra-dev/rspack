it("should add all modules headers info above modules", () => {
    const fs = require("fs");
    const path = require("path")
    const content = fs.readFileSync(path.join(__dirname, "sut.js"), "utf-8");

    expect(content).toContain(`!*** ./sut.js ***!`)
    expect(content).toContain(`!*** ./util.js ***!`)
    expect(content).toContain(`!*** ./cjs.js ***!`)
})