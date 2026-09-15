const generated = /** @type {string} */ (
  require('fs').readFileSync(__filename, 'utf-8')
);

it('should transform optional chaining for an older Baseline target', () => {
  // START:A
  const value = { nested: { answer: 42 } };
  const answer = value?.nested?.answer;
  // END:A
  const block = generated.match(/\/\/ START:A([\s\S]*)\/\/ END:A/)[1];

  expect(answer).toBe(42);
  expect(block).not.toContain('?.');
  expect(block).toContain('=== null');
});
