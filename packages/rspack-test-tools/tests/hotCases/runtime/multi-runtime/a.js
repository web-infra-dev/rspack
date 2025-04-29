import * as classes from "./a.module.css"

module.hot.accept('./a.module.css')

it("runtime a hmr", (done) => {
  expect(typeof global["webpackHotUpdate_main"] === "function").toBe(true);
  expect(classes.a1).toBeDefined();
  const a2 = "a2";
  expect(classes[a2]).not.toBeDefined();
	NEXT(require("../../update")(done, true, () => {
    expect(classes.a1).toBeDefined();
    expect(classes[a2]).toBeDefined();
    done()
	}));
});
