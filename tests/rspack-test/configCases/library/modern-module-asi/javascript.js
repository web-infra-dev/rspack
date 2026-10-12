import { value } from './no-semicolon.js';

(() => {
  it('should terminate JavaScript after a trailing line comment', () => {
    expect(value.version).toBe('1.2.3');
  });
})();
