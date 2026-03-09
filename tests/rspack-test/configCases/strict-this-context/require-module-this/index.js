it("should generate optimized code or correct code based on strictThisContextOnImports", () => {
    const value = require("./module").that().value;
    expect(value).toBe(STRICT_THIS_CONTEXT_ON_IMPORTS ? 42 : undefined);
});
