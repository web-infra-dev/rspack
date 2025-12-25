import './call'; // this should have no side effects at all
import { foo20 } from './re-export'

const _ = foo20()

it('should have no side effects for all', () => {
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(1);
})
