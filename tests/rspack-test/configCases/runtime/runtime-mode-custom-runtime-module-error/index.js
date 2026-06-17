export default 1;

it('should support custom runtime modules in rspack runtime mode', () => {
  expect(globalThis.__custom_runtime_module_value__).toBe(1);
  expect(globalThis.__custom_runtime_module_shadow__).toBe(2);
});
