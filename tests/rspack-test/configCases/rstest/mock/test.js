const path = require('path');
const fs = require('fs');

const syncMockFactoryFile = path.resolve(__dirname, 'bundle1.js');

it('mocked modules should be hoisted before user code', () => {
  const content = fs.readFileSync(syncMockFactoryFile, 'utf-8');
  const firstMockHoist = content.indexOf('[Rstest mock hoist]');
  const afterTopOfFile = content.indexOf('TOP_OF_FILE');

  expect(firstMockHoist).toBeGreaterThan(-1);
  expect(afterTopOfFile).toBeGreaterThan(-1);
  expect(afterTopOfFile).toBeGreaterThan(firstMockHoist);
});

it('should not wrap synchronous mock factories as async modules', () => {
  const content = fs.readFileSync(syncMockFactoryFile, 'utf-8');

  expect(content.includes('__rspack_async_done')).toBe(false);
  expect(content.includes('__webpack_require__.a(')).toBe(false);
});
