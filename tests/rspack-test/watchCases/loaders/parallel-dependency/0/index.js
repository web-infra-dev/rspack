const workerReturn = require('./worker-return');
const workerYield = require('./worker-yield');

it('should preserve dependencies from uncached parallel loaders', () => {
  const step = +WATCH_STEP;
  expect(workerReturn).toBe(`return-${step}`);
  expect(workerYield).toBe(`yield-${step}`);
});
