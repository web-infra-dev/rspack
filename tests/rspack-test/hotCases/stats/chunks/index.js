import value from './file'

it("should correctly handle hot module replacement", done => {
    expect(value).toBe(1);
    NEXT(require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
        expect(value).toBe(2);
        done();
    }));
});

module.hot.accept("./file");
