import value from './file'

it("should be pass", done => {
    expect(value).toBe(1);
    NEXT(require("../../update")(done, true, () => {
        expect(value).toBe(2);
        done();
    }));
});

module.hot.accept("./file");
