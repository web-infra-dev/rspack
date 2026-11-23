import value from './value';

it('should rebuild a cached module when rebuildModule is called in finishMake', async () => {
  expect(value).toBe(`rebuilt-${COMPILER_INDEX}`);
  if (COMPILER_INDEX === 0) await NEXT_START();
});
