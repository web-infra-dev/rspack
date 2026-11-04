import { createRequire } from 'node:module';
import path from 'node:path';

const require = createRequire(import.meta.url);
const source = path.resolve(
  path.dirname(require.resolve('@rspack/core/package.json')),
  '../../tests/rspack-test/cacheCases/rstest/preserve-direct-imports',
);
const { createConfig } = require(path.join(source, 'rspack.config.ts'));
export default function (_env: unknown, { tempPath }: { tempPath: string }) {
  return {
    ...createConfig(true, source),
    entry: path.join(source, 'index.js'),
    resolve: {
      alias: { './generation': path.join(tempPath, 'generation.js') },
    },
  };
}
