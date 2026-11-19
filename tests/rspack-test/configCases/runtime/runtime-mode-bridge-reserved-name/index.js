export const __rspack_context = 42;
export const __webpack_module_cache__ = 43;

it("keeps module bindings with the same names as the bridge variables", () => {
  expect(__rspack_context).toBe(42);
  expect(__webpack_module_cache__).toBe(43);
  expect(__webpack_public_path__).toBe("/plugin/");
});
