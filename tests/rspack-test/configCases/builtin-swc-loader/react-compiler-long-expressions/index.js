import { build as plain } from './plain';
import { build as optedOut } from './opt-out';
import { build as withReact, Component } from './with-react.jsx';

it('should preserve long expressions with React Compiler enabled', () => {
  const indices = Array.from({ length: 1000 }, (_, index) => index);
  const expected = indices.map(index => `a${index}${index}`).join('');

  for (const build of [plain, optedOut, withReact]) {
    const calls = [];
    const result = build(index => {
      calls.push(index);
      return index;
    });

    expect(result).toBe(expected);
    expect(calls).toEqual(indices);
  }

  expect(typeof Component).toBe('function');
});
