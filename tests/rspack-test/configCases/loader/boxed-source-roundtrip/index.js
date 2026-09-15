import normal from './input.js?normal';
import pitched from './input.js?pitch';

it('preserves binary bytes, maps and additional data across native loaders', () => {
  expect(normal).toEqual({ hex: '00fffe800a', value: 42 });
  expect(pitched).toEqual(normal);
});
