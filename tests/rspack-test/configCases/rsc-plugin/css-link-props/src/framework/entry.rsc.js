import { ReadableStream } from 'node:stream/web';
import { renderToReadableStream } from 'react-server-dom-rspack/server';
import { App, getCssNodes } from '../App';
import { Client } from '../Client';

globalThis.ReadableStream = ReadableStream;

export const renderRscStream = () => {
  return renderToReadableStream(<App />);
};

const readStream = async stream => {
  const reader = stream.getReader();
  const decoder = new TextDecoder();
  let result = '';
  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      result += decoder.decode();
      return result;
    }
    result += decoder.decode(value, { stream: true });
  }
};

it('should expose configured CSS link props in the RSC runtime manifest', () => {
  const manifest = __rspack_rsc_manifest__;

  expect(manifest.cssLinkProps).toEqual({
    as: 'style',
    'data-rspack-rsc': 'enabled',
  });
  expect(manifest.cssLinkProps).not.toHaveProperty('precedence');
});

it('should apply configured CSS link props only through loadCss', () => {
  const [linkElement] = getCssNodes();

  expect(linkElement.type).toBe('link');
  expect(linkElement.props).toEqual(
    expect.objectContaining({
      as: 'style',
      'data-rspack-rsc': 'enabled',
      rel: 'stylesheet',
      href: expect.stringMatching(/\.css$/),
    }),
  );
  expect(linkElement.key).toMatch(/\.css$/);
  expect(linkElement.props).not.toHaveProperty('precedence');
});

it('should export use client values as client references', () => {
  expect(Client.$$typeof).toBe(Symbol.for('react.client.reference'));
});

it('should preinit client CSS in the RSC payload', async () => {
  const stream = renderToReadableStream(<App />);
  const payload = await readStream(stream);
  const clientCssFiles = Object.values(
    __rspack_rsc_manifest__.clientManifest,
  ).flatMap(entry => entry.cssFiles || []);

  expect(clientCssFiles).toEqual([expect.stringMatching(/\.css$/)]);
  expect(payload).toContain(':HS');
  expect(payload).toContain(clientCssFiles[0]);
  expect(payload).toContain('rspack-rsc/client-reference');
});
