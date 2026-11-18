import step from './step';

it('should rebuild when another module changes', () => {
  expect(step).toBe(Number(WATCH_STEP));
});
