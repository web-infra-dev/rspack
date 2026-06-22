import { renderToReadableStream } from 'react-server-dom-rspack/server';
import { App } from '../App';
import { Client } from '../Client';

export const renderRscStream = () => {
  return renderToReadableStream(<App />);
};

it('should expose configured CSS link props in the RSC runtime manifest', () => {
  const manifest = __rspack_rsc_manifest__;

  expect(manifest.cssLinkProps).toEqual({
    as: 'style',
    'data-rspack-rsc': 'enabled',
  });
  expect(manifest.cssLinkProps).not.toHaveProperty('precedence');
});

it('should apply configured CSS link props to generated link elements', () => {
  const clientElement = Client({});
  const [resourceElements] = clientElement.props.children;
  const [linkElement] = resourceElements;

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
