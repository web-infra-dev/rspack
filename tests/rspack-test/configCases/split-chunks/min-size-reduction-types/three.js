import './index';

it('keeps the third consumer executable', async () => {
  expect((await import('./legacy')).default).toBe('shared:legacy');
});
