it("should not include foo.js", async () => {
    let a1 = 'a1';
    expect(require(`./${process.env.DIR}/` + a1)).toBe("a1");
});
