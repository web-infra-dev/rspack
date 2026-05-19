import './consumer';

it('should drop aliased and default deferred pure imports', () => {
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(1);
});
