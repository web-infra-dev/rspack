const __rspack_module = "user rspack module";
export const moduleId = __webpack_module__.id;

it("keeps user bindings when the rspack module argument name conflicts", function () {
  expect(__rspack_module).toBe("user rspack module");
  expect(moduleId).toBeDefined();
});
