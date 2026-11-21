it('loads the fallback for each resolved resource query', async () => {
  const [first, second] = await Promise.all([import('first'), import('second')]);
  expect([first.default, second.default]).toEqual(['?a', '?b']);
});
