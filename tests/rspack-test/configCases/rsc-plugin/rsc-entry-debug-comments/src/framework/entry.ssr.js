import { createFromReadableStream } from 'react-server-dom-rspack/client';
import { renderRscStream } from './entry.rsc';

export const render = async () => {
  const rscStream = await renderRscStream();
  return createFromReadableStream(rscStream);
};
