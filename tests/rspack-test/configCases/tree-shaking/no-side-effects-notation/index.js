import './call'; // this should have no side effects at all
import { foo16, v } from './re-export'

const unused1 = foo16();
const unused2 = v;
it('should have no side effects for all', () => {
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(1);
})
