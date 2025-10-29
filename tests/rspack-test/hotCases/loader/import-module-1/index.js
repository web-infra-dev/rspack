import a from "./loader.js!./a";

it("module and its loader-referencing module should update in right order", async () => {
  expect(a).toBe(1);
  await NEXT_HMR();
  expect(a).toBe(2);
});

module.hot.accept('./loader.js!./a');
