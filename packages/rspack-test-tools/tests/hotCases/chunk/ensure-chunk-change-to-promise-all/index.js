import { test } from "./chunk"

it("should still works when ensure chunk causes the parent chunk change", (done) => {
  test(0).then((React) => {
    expect(React).toBe(42);
    import.meta.webpackHot.accept("./chunk", () => {
      test(1).then((Vue) => {
        expect(Vue).toBe(43);
        done()
      })
    });
    NEXT(require("../../update")(done));
  }).catch(done)
});
