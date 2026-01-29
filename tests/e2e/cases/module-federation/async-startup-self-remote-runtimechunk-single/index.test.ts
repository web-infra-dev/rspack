import path from 'node:path';
import { test, expect } from '@/fixtures';

const readOutputFile = (rspack: any, file: string) => {
  return rspack.compiler.outputFileSystem.readFileSync(
    path.join(rspack.outDir, file),
    'utf-8',
  );
};

test('async startup with runtimeChunk single wires mfAsyncStartup', async ({
  page,
  rspack,
}) => {
  await expect(page.locator('#root')).toHaveText('Remote Module Loaded');

  const mainContent = readOutputFile(rspack, 'static/js/main.js');
  const runtimeContent = readOutputFile(rspack, 'static/js/runtime.js');
  expect(mainContent).toContain('__webpack_require__.X(');
  expect(runtimeContent).toContain('mfAsyncStartup');
});
