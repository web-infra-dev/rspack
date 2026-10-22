import value from './changing.txt';

it('should refresh file system dependencies after HMR', async () => {
  expect(value).toBe('a');
  await NEXT_HMR({ hot: import.meta.webpackHot });
  expect(value).toBe('b');
  await NEXT_HMR({ hot: import.meta.webpackHot });
  expect(value).toBe('a');
});

import.meta.webpackHot.accept('./changing.txt');
