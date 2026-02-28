
it("should correctly compile a dynamic entry with `dependOn` when the entry file is created in watch mode", async () => {
  await import("./foo").then(mod => {
        expect(mod.default).toBe("foo");
    });
});
