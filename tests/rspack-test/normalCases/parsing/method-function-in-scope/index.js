import { f as a } from "./utils";

function g() {
  return {
    f1() {
      var a = 2;
      return a;
    },
    f2() {
      return a();
    },
  };
}

it("a should refer to f in utils.js", () => {
  expect(g().f2()).toBe(1);
});
