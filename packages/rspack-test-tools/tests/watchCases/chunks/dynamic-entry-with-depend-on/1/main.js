
it("should correctly compile a dynamic entry with `dependOn` when the entry file is created in watch mode", (done) => {
    import("./foo").then(mod => {
        expect(mod.default).toBe("foo");
        done();
    });
});
