import { testInitialChunks } from './testCases';

for (const mode of ['extract', 'native', 'esm'] as const) {
  testInitialChunks(mode);
}
