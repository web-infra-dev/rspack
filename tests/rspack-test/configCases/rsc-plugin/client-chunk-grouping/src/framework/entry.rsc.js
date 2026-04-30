import { renderToReadableStream } from 'react-server-dom-rspack/server';
import { Root } from '../Root';

export const renderRscStream = () => {
  return renderToReadableStream(<Root />);
};

it('should build the RSC client chunk grouping fixture', () => {
  expect(typeof renderRscStream).toBe('function');
});
