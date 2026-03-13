import * as reexport from "./reexport";

it("should generate optimized code or correct code based on strictThisContextOnImports", () => {
	expect(reexport.reexported.that().value).toBe(STRICT_THIS_CONTEXT_ON_IMPORTS ? "reexported" : undefined);
    expect(reexport.reexported.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["that", "usedExports"]);
});
