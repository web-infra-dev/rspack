import { expect, test } from '@/fixtures';
import path from 'node:path';

function readMain(rspack: any) {
  const file = rspack.compiler.outputFileSystem
    .readdirSync(rspack.outDir)
    .find(
      (file: string) =>
        !file.includes('hot-update') &&
        (file.endsWith('.js') || file.endsWith('.mjs')),
    );
  return rspack.compiler.outputFileSystem
    .readFileSync(path.join(rspack.outDir, file))
    .toString();
}

async function expect_inlined_reexport(rspack: any, value: string) {
  await expect(async () => {
    expect(readMain(rspack)).toContain(`"value",()=>${value}`);
  }).toPass();
}

test('should compile', async ({ fileAction, rspack }) => {
  // rspack.compiler.__sharedObj is injected by plugin in rspack.config.js
  await expect_inlined_reexport(rspack, '2');

  rspack.compiler.__sharedObj.time++;
  fileAction.updateFile('file.js', (content) => content.replace('1', '2'));
  await rspack.waitingForBuild();

  await expect_inlined_reexport(rspack, '4');

  rspack.compiler.__sharedObj.time++;
  fileAction.updateFile('file.js', (content) => content.replace('2', '3'));
  await rspack.waitingForBuild();

  await expect_inlined_reexport(rspack, '6');
});
