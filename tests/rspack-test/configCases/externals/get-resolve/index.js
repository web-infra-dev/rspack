global.cjs = "cjs";
global.esm = "esm";

it("should resolve right file", async () => {
    expect(require("foo")).toBe("cjs");
    expect((await import("foo")).default).toBe("esm");
})

