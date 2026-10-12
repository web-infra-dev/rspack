import data from './array.json' with { type: 'json' };

[0];

it('should not turn a following array expression into property access', () => {
  expect(data).toEqual(['value']);
});
