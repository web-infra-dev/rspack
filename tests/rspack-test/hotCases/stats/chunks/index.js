import value from './file'

it("should correctly handle hot module replacement", async () => {
    expect(value).toBe(1);
    await NEXT_HMR();
    expect(value).toBe(2);
});

module.hot.accept("./file");
