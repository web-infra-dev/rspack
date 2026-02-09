// Framework conventions (arbitrary choices for this demo):
// - Use `x-rsc-action` header to pass server action ID
const HEADER_ACTION_ID = 'x-rsc-action';
const RSC_PAYLOAD_ACCEPT = 'text/x-component';

// Parsed request information used to route between RSC/SSR rendering and action handling.
// Created by parseRenderRequest() from incoming HTTP requests.
type RenderRequest = {
  isRsc: boolean; // true if request should return RSC payload
  isAction: boolean; // true if this is a server action call (POST request)
  actionId?: string; // server action ID from x-rsc-action header
  request: Request; // normalized Request
  url: URL; // normalized URL with
};

export function createRscRenderRequest(
  urlString: string,
  action?: { id: string; body: BodyInit },
): Request {
  const url = new URL(urlString, location.origin);
  const headers = new Headers();
  if (action) {
    headers.set(HEADER_ACTION_ID, action.id);
  } else {
    headers.set('Accept', RSC_PAYLOAD_ACCEPT);
  }
  return new Request(url.toString(), {
    method: action ? 'POST' : 'GET',
    headers,
    body: action?.body,
  });
}

export function parseRenderRequest(request: Request): RenderRequest {
  const url = new URL(request.url);
  if (request.method === 'POST') {
    const actionId = request.headers.get(HEADER_ACTION_ID);
    if (!actionId) {
      throw new Error('Missing action id header for RSC action request');
    }
    return {
      isRsc: true,
      isAction: true,
      actionId,
      request: new Request(url, request),
      url,
    };
  }
  if (request.headers.get('Accept')?.includes('text/html')) {
    return {
      isRsc: false,
      isAction: false,
      request,
      url,
    };
  }
  return {
    isRsc: true,
    isAction: false,
    request: new Request(url, request),
    url,
  };
}
