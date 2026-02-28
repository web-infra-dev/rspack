import { createFromReadableStream } from 'react-server-dom-rspack/client';
import { renderRscStream } from './entry.rsc';

export const renderHTML = async () => {
  // In real SSR, the HTML renderer would consume the RSC stream.
  // For this test case we just ensure the pipeline can be invoked.
  const rscStream = await renderRscStream();
  return createFromReadableStream(rscStream);
};
