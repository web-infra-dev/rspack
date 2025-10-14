import { test } from "./chunk"

it("should still works when ensure chunk causes the parent chunk change", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
  test(0).then((React) => {
    expect(React).toBe(42);
    import.meta.webpackHot.accept("./chunk", () => {
      test(1).then((Vue) => {
        expect(Vue).toBe(43);
        done()
      })
    });
    NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
  }).catch(done)
}));
