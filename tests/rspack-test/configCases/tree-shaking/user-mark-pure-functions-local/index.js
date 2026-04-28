import './dep';

it('should treat non-exported local fn marked via pureFunctions as pure', () => {
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(1);
});
