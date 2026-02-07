import React from 'react';
import type { ReactFormState } from 'react-dom/client';
import { renderToReadableStream } from 'react-dom/server';
import { createFromReadableStream } from 'react-server-dom-rspack/client';
import { injectRSCPayload } from 'rsc-html-stream/server';
import type { RscPayload } from './entry.rsc';

export async function renderHTML(
  rscStream: ReadableStream<Uint8Array>,
  options: {
    bootstrapScripts?: string[];
    formState?: ReactFormState;
    nonce?: string;
    debugNojs?: boolean;
  },
) {
  // duplicate one RSC stream into two.
  // - one for SSR (ReactClient.createFromReadableStream below)
  // - another for browser hydration payload by injecting <script>...FLIGHT_DATA...</script>.
  const [rscStream1, rscStream2] = rscStream.tee();

  // deserialize RSC stream back to React VDOM
  let payload: Promise<RscPayload>;
  function SsrRoot() {
    // deserialization needs to be kicked off inside ReactDOMServer context
    // for ReactDomServer preinit/preloading to work
    payload ??= createFromReadableStream<RscPayload>(rscStream1);
    return React.use(payload).root;
  }

  // render html (traditional SSR)
  let htmlStream: ReadableStream<Uint8Array>;
  let status: number | undefined;
  try {
    htmlStream = await renderToReadableStream(<SsrRoot />, {
      bootstrapScripts: options?.debugNojs
        ? undefined
        : options.bootstrapScripts,
      nonce: options?.nonce,
      formState: options?.formState,
    });
  } catch {
    // fallback to render an empty shell and run pure CSR on browser,
    // which can replay server component error and trigger error boundary.
    status = 500;
    htmlStream = await renderToReadableStream(
      <html lang="en">
        <body>
          <noscript>Internal Server Error: SSR failed</noscript>
        </body>
      </html>,
      {
        nonce: options?.nonce,
      },
    );
  }

  let responseStream: ReadableStream<Uint8Array> = htmlStream;
  if (!options?.debugNojs) {
    // initial RSC stream is injected in HTML stream as <script>...FLIGHT_DATA...</script>
    responseStream = responseStream.pipeThrough(
      injectRSCPayload(rscStream2, {
        nonce: options?.nonce,
      }),
    );
  }

  return { stream: responseStream, status };
}
