import value from 'http://integrity.example/module.js';

it('uses the content accepted by the HTTP lockfile policy', () => {
  expect(value).toBe(EXPECTED_VALUE);
});
