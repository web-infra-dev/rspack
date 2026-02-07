import type { IncomingMessage, ServerResponse } from 'node:http';
import { parentPort } from 'node:worker_threads';
import express from 'express';
import type React from 'react';
import type { ReactFormState } from 'react-dom/client';
import {
  createTemporaryReferenceSet,
  decodeAction,
  decodeFormState,
  decodeReply,
  loadServerAction,
  renderToReadableStream,
  type ServerEntry,
  type TemporaryReferenceSet,
} from 'react-server-dom-rspack/server.node';
import { toNodeHandler } from 'srvx/node';
import { renderHTML } from './entry.ssr.tsx';
import { parseRenderRequest } from './request.tsx';

// The schema of payload which is serialized into RSC stream on rsc environment
// and deserialized on ssr/client environments.
export type RscPayload = {
  // this demo renders/serializes/deserizlies entire root html element
  // but this mechanism can be changed to render/fetch different parts of components
  // based on your own route conventions.
  root: React.ReactNode;
  // server action return value of non-progressive enhancement case
  returnValue?: { ok: boolean; data: unknown };
  // server action form state (e.g. useActionState) of progressive enhancement case
  formState?: ReactFormState;
};

async function handleRequest({
  request,
  getRoot,
  bootstrapScripts,
  nonce,
}: {
  request: Request;
  getRoot: () => React.ReactNode;
  bootstrapScripts?: string[];
  nonce?: string;
}): Promise<Response> {
  // differentiate RSC, SSR, action, etc.
  const renderRequest = parseRenderRequest(request);
  request = renderRequest.request;

  // handle server function request
  let returnValue: RscPayload['returnValue'] | undefined;
  let formState: ReactFormState | undefined;
  let temporaryReferences: TemporaryReferenceSet | undefined;
  let actionStatus: number | undefined;
  if (renderRequest.isAction === true) {
    if (renderRequest.actionId) {
      // action is called via `ReactClient.setServerCallback`.
      const contentType = request.headers.get('content-type');
      const body = contentType?.startsWith('multipart/form-data')
        ? await request.formData()
        : await request.text();
      temporaryReferences = createTemporaryReferenceSet();
      const args = await decodeReply(body, { temporaryReferences });
      const action = loadServerAction(renderRequest.actionId);
      try {
        const data = await action.apply(null, args);
        returnValue = { ok: true, data };
      } catch (e) {
        returnValue = { ok: false, data: e };
        actionStatus = 500;
      }
    } else {
      // otherwise server function is called via `<form action={...}>`
      // before hydration (e.g. when javascript is disabled).
      // aka progressive enhancement.
      const formData = await request.formData();
      const decodedAction = await decodeAction(formData);
      try {
        const result = await decodedAction();
        formState = (await decodeFormState(result, formData)) as ReactFormState;
      } catch {
        // there's no single general obvious way to surface this error,
        // so explicitly return classic 500 response.
        return new Response('Internal Server Error: server action failed', {
          status: 500,
        });
      }
    }
  }

  const rscPayload: RscPayload = { root: getRoot(), formState, returnValue };
  const rscOptions = { temporaryReferences };
  const rscStream = renderToReadableStream(rscPayload, rscOptions);

  // Respond RSC stream without HTML rendering as decided by `RenderRequest`
  if (renderRequest.isRsc) {
    return new Response(rscStream, {
      status: actionStatus,
      headers: {
        'content-type': 'text/x-component;charset=utf-8',
      },
    });
  }

  // Delegate to SSR environment for html rendering.
  const ssrResult = await renderHTML(rscStream, {
    bootstrapScripts,
    formState,
    nonce,
    // allow quick simulation of javascript disabled browser
    debugNojs: renderRequest.url.searchParams.has('__nojs'),
  });

  // respond html
  return new Response(ssrResult.stream, {
    status: ssrResult.status,
    headers: {
      'content-type': 'text/html;charset=utf-8',
    },
  });
}

async function handler(request: Request, id?: number): Promise<Response> {
  const { Todos } = await import('../Todos.tsx');
  const serverEntry = Todos as ServerEntry<typeof Todos>;
  const nonce = !process.env.NO_CSP ? crypto.randomUUID() : undefined;
  const nonceMeta = nonce && <meta property="csp-nonce" nonce={nonce} />;
  const root = (
    <>
      {nonceMeta}
      {serverEntry.entryCssFiles
        ? serverEntry.entryCssFiles.map((href) => (
            <link
              key={href}
              rel="stylesheet"
              href={href}
              precedence="default"
            ></link>
          ))
        : null}
      <Todos id={id} />
    </>
  );
  const response = await handleRequest({
    request,
    getRoot: () => root,
    bootstrapScripts: serverEntry.entryJsFiles,
    nonce,
  });
  if (nonce && response.headers.get('content-type')?.includes('text/html')) {
    const cspValue = [
      `default-src 'self';`,
      // `unsafe-eval` is required during dev since React uses eval for findSourceMapURL feature
      `script-src 'self' 'nonce-${nonce}' ${
        process.env.NODE_ENV === 'development' ? `'unsafe-eval'` : ``
      };`,
      `style-src 'self' 'unsafe-inline';`,
      `img-src 'self' data:;`,
      // allow blob: worker for Vite server ping shared worker
      import.meta.webpackHot && `worker-src 'self' blob:;`,
    ]
      .filter(Boolean)
      .join('');
    response.headers.set('content-security-policy', cspValue);
  }
  return response;
}

const fetch = (
  req: IncomingMessage,
  res: ServerResponse<IncomingMessage>,
  id?: number,
) => toNodeHandler((req) => handler(req, id))(req, res);

async function nodeHandler(
  req: IncomingMessage,
  res: ServerResponse<IncomingMessage>,
  next: () => void,
) {
  // Handle GET requests to root path
  if (req.method === 'GET' && req.url === '/') {
    await fetch(req, res);
    return;
  }

  // Handle POST requests to root path
  if (req.method === 'POST' && req.url === '/') {
    await fetch(req, res);
    return;
  }

  // Handle GET requests to /todos/:id
  if (req.method === 'GET' && req.url?.startsWith('/todos/')) {
    const id = req.url.split('/')[2];
    if (id) {
      await fetch(req, res, Number(id));
      return;
    }
  }

  // Handle POST requests to /todos/:id
  if (req.method === 'POST' && req.url?.startsWith('/todos/')) {
    const id = req.url.split('/')[2];
    if (id) {
      await fetch(req, res, Number(id));
      return;
    }
  }

  next();
}

const app = express();

app.use(nodeHandler);

app.use(express.static(import.meta.dirname));

app.listen(3000, () => {
  if (parentPort) {
    parentPort.postMessage({ type: 'ready' });
  }

  console.log('Server is running on http://localhost:3000');
});
if (import.meta.webpackHot) {
  import.meta.webpackHot.accept();
}
