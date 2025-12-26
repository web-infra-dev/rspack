import './call'; // this should have no side effects at all
import './re-export'

it('should have no side effects for all', () => {
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(1);
})
