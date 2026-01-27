import { createSnapshotSerializer } from 'path-serializer';

// 1. escapeEOL \r\n -> \n
// 2. replace <RSPACK_ROOT> etc
// 3. transform win32 sep
const placeholderSerializer = createSnapshotSerializer({
  root: __dirname.includes('node_modules')
    ? // Use `process.cwd()` when using outside Rspack
      process.cwd()
    : __ROOT_PATH__,
  replace: [
    {
      match: __RSPACK_TEST_TOOLS_PATH__,
      mark: 'test_tools_root',
    },
    {
      match: __TEST_PATH__,
      mark: 'test_root',
    },
    {
      match: __RSPACK_PATH__,
      mark: 'rspack_root',
    },
    {
      match: /:\d+:\d+-\d+:\d+/g,
      mark: 'line_col_range',
    },
    {
      match: /:\d+:\d+/g,
      mark: 'line_col',
    },
  ],
  features: {
    replaceWorkspace: false,
    addDoubleQuotes: false,
    escapeDoubleQuotes: false,
  },
});

export const normalizePlaceholder = placeholderSerializer.serialize;
