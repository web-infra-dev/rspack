import './dep';

it('should accept destructured top-level bindings as pureFunctions', () => {
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(1);
});
