import { value } from './value';
import path from 'external-value';
import ignored from 'ignored';
const context = require.context('./context', false, /\.js$/);
it('uses JS Module classes with a native handle', () => {
  expect(value).toBe(42);
  expect(path.basename('/a/b')).toBe('b');
  expect(ignored).toEqual({});
  expect(context('./item.js')).toBe(7);
});
