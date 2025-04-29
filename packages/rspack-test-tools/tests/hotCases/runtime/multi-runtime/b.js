import * as classes from "./b.module.css"

it("runtime a hmr", (done) => {
  expect(typeof global["webpackHotUpdate_b"] === "function").toBe(true);
  expect(classes.b1).toBeDefined();
});
