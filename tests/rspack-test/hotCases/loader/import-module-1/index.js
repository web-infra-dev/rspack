import a from "./loader.js!./a";

it("module and its loader-referencing module should update in right order", (done) => {
  expect(a).toBe(1);
  NEXT(
    require('../../update')(done, true, () => {
      expect(a).toBe(2);
      done();
    }),
  );
});

module.hot.accept('./loader.js!./a');