it("should handle loader error with empty stack trace", () => {
    expect(() => {
        require("./lib");
    }).toThrow("Failed to load");
});
