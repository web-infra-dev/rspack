import path from 'node:path';
import { renderToReadableStream } from 'react-server-dom-rspack/server';
import { App } from '../App';

const fs = __non_webpack_require__('node:fs');

const RSC_ENTRY_MODULE_HEADER = '"rspack/rsc-entry?name=main"() {';

const readBundle = (filename) => {
  return fs.readFileSync(path.join(__dirname, filename), 'utf-8');
};

const extractRscEntryModuleContent = (source) => {
  const start = source.indexOf(RSC_ENTRY_MODULE_HEADER);
  expect(start).toBeGreaterThanOrEqual(0);

  const contentStart = start + RSC_ENTRY_MODULE_HEADER.length;
  const contentEnd = source.indexOf('\n},', contentStart);
  expect(contentEnd).toBeGreaterThanOrEqual(0);

  return source.slice(contentStart, contentEnd).trim();
};

export const renderRscStream = () => {
  return renderToReadableStream(<App />);
};

it('should emit readable RSC entry debug comments by default', () => {
  const snapshot = `${[
    'server',
    extractRscEntryModuleContent(readBundle('server.js')),
    '',
    'client',
    extractRscEntryModuleContent(readBundle('client.js')),
  ].join('\n')}\n`;

  expect(snapshot).toMatchFileSnapshotSync(
    path.join(__SNAPSHOT__, 'rsc-entry-debug-comments.txt'),
  );
});
