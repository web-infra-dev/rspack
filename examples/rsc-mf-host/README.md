# Rspack React Server Components Example

This example is a server-driven app built with Rspack and React Server Components (RSC). In this setup, routing happens on the server, delivering HTML on initial page load, and client side rendering on subsequent navigations. It also demonstrates React Server Actions to perform mutations, both by calling as a function and as the target of an HTML form.

## Setup

The example consists of the following main files:

### server.js

This is the development server setup using Express, Rspack middleware and webpack-hot-middleware for HMR. It configures two Rspack compilation targets: one for the client bundle (web target) and one for the RSC server bundle (node target). The server imports the compiled RSC entry module and delegates requests to it.

The Rspack configuration that defines three build targets:

1. **Client bundle** (web target): Compiles `src/framework/entry.client.tsx` with React Refresh and HMR support
2. **RSC server bundle** (node target): Compiles `src/framework/entry.rsc.tsx` with RSC layer support using `rspack.experiments.rsc` plugins
3. Both configurations use `builtin:swc-loader` with `rspackExperiments.reactServerComponents: true` to enable RSC support

The RSC configuration uses layers (`Layers.rsc` and `Layers.ssr`) to differentiate between server component code and SSR code, with appropriate resolve conditions (`react-server`) for RSC modules.

### src/Todos.tsx

This is the entry React Server Component that renders the root `<html>` element, server content, and any client components. It is marked with the `"use server-entry"` directive, which indicates this is an entry point for the server component tree.

### src/framework/entry.client.tsx

This is the main client entrypoint that hydrates the initial page and handles client-side routing. It uses `react-server-dom-rspack/client.browser` to deserialize RSC payloads into React VDOM. The client intercepts navigation events (via `popstate` and `history.pushState`) and re-fetches RSC payloads for client-side transitions. It also registers a server callback using `setServerCallback` to handle server action calls.

### src/actions.ts

This is a server actions file. Functions exported by this file can be imported from the client and called to send data to the server for processing. It is marked using the `"use server"` directive. Rspack's RSC plugin detects this directive and places these actions into the server bundle while creating proxy modules on the client that communicate with the server via the handler registered in `entry.client.tsx`.

Currently, server actions must be defined in a separate file. Inline server actions (e.g. `"use server"` inside a function) are not yet supported.

### src/framework/entry.rsc.tsx

This module handles RSC rendering and server action execution on the server using `react-server-dom-rspack/server.node`. It exports a request handler that:
- Differentiates between RSC fetch requests, SSR requests, and action calls
- Handles server actions by decoding the request and executing the action
- Renders the React tree to an RSC stream using `renderToReadableStream`
- Delegates to SSR for initial HTML rendering or returns raw RSC payload for client-side navigation

### src/framework/entry.ssr.tsx

This module performs server-side rendering (SSR) of React components. It receives an RSC stream, deserializes it back into React VDOM using `createFromReadableStream` from `react-server-dom-rspack/client`, then renders it to HTML using `react-dom/server`. The RSC payload is also injected into the HTML as a script tag for client-side hydration using `rsc-html-stream`.

### src/TodoItem.tsx and src/Dialog.tsx

These are client components. `<TodoItem>` renders a todo list item, and uses server actions and `useOptimistic` to implement the checkbox and remove buttons. `Dialog.tsx` renders a dialog component using client APIs, and accepts the create todo form (which is a server component) as children.

## Initial HTML rendering

The flow of initial rendering starts on the server.

### Server

The server uses Express to handle routing. When a route handler is called, it invokes the handler from `entry.rsc.tsx` which:

1. Parses the request to determine if it's an RSC fetch, action call, or initial HTML request
2. Renders the React component tree to an RSC stream using `renderToReadableStream` from `react-server-dom-rspack/server.node`
3. For initial HTML requests, delegates to `entry.ssr.tsx` which:
   - Deserializes the RSC stream back into React VDOM using `createFromReadableStream` from `react-server-dom-rspack/client`
   - Renders the VDOM to HTML using `react-dom/server`
   - Injects the RSC payload into the HTML stream as a script tag for client hydration

This approach allows the same RSC stream to be used for both SSR and client hydration, reducing redundant work.

### Client

To hydrate the initial page, the client calls `createFromReadableStream` from `react-server-dom-rspack/client.browser` to deserialize the RSC payload embedded in the initial HTML (via `rsc-html-stream`). The deserialized payload is then hydrated using `hydrateRoot` from `react-dom/client`, making the page interactive.

## Client side routing

The client includes a simple router in `entry.client.tsx`, allowing subsequent navigations after the initial page load to maintain client state without reloading the full HTML page.

### Client

The client listens for navigation events using `popstate` (browser back/forward buttons) and intercepts `history.pushState` calls. To perform a navigation:

1. Call `createFromFetch` from `react-server-dom-rspack/client.browser` to fetch a new RSC payload from the server with the appropriate `Accept` header
2. Update the component state with the new payload, triggering a React transition to re-render the page
3. Push the new URL to the browser's history if needed

These steps can be customized as needed for your server setup, e.g. using a more sophisticated client side router, or adding authentication headers.

### Server

The server handles fetch requests for RSC payloads using the same request handler in `entry.rsc.tsx`. When the request is identified as an RSC fetch (based on request headers), `renderToReadableStream` serializes the component tree into an RSC payload and returns it with the `text/x-component` content type, skipping the SSR step.

## Server actions

Server actions allow the client to call the server to perform mutations and other actions. There are two ways server actions can be called: by calling an action function from the client, or by submitting an HTML form (progressive enhancement).

### Client

When a server action is called, the client sends a request to the server using the `callServer` callback registered via `setServerCallback` in `entry.client.tsx`. When a server action proxy function generated by Rspack is called on the client, this handler will be invoked with the id of the action and the arguments to pass to it.

1. Create a request object with the action ID and encode the arguments using `encodeReply` from `react-server-dom-rspack/client.browser`
2. Call `createFromFetch` to fetch the response, which includes both the new component tree and the return value of the server action
3. Update the page state with the returned payload, triggering a re-render
4. Extract and return the result of the server action from the payload

These steps can be customized as needed for your server setup, e.g. adding authentication headers.

### Server

When a server action request is received in `entry.rsc.tsx`, the server performs the following steps:

1. Parse the request to extract the action ID and decode the arguments using `decodeReply` from `react-server-dom-rspack/server.node`
2. Load and execute the server action using `loadServerAction`
3. Capture the return value (or error) from the action
4. Render the component tree to an RSC payload using `renderToReadableStream`, including the action return value in the response
5. Return the RSC payload to the client

For progressive enhancement (form submissions before JavaScript loads), the server decodes the form data using `decodeAction` and `decodeFormState`, executes the action, and returns the form state in the RSC payload for proper hydration.
