import './consumer';

it('should drop pure imported call when marked on consumer', () => {
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(1);
});
