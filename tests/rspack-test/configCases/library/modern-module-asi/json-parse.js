import pkg from './large.json' with { type: 'json' };

(() => {
  it('should separate JSON.parse output from a following call expression', () => {
    expect(pkg.version).toBe('1.2.3-with-a-long-version-string');
  });
})();
