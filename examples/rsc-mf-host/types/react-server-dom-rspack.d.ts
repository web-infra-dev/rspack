declare module 'react-server-dom-rspack/client' {
  import type { Options } from 'react-server-dom-rspack/client.edge';

  export { Options };

  type TemporaryReferenceSet = Map<string, unknown>;

  export type CallServerCallback = (
    id: string,
    args: unknown[],
  ) => Promise<unknown>;

  export type EncodeFormActionCallback = <A>(
    id: any,
    args: Promise<A>,
  ) => ReactCustomFormAction;

  export type ReactCustomFormAction = {
    name?: string;
    action?: string;
    encType?: string;
    method?: string;
    target?: string;
    data?: null | FormData;
  };

  export type FindSourceMapURLCallback = (
    fileName: string,
    environmentName: string,
  ) => null | string;

  export function createFromFetch<T>(
    promiseForResponse: Promise<Response>,
    options?: Options,
  ): Promise<T>;

  export function createFromReadableStream<T>(
    stream: ReadableStream,
    options?: Options,
  ): Promise<T>;

  export function createServerReference(
    id: string,
    callServer: CallServerCallback,
    encodeFormAction: EncodeFormActionCallback | undefined,
    findSourceMapURL: FindSourceMapURLCallback | undefined, // DEV-only
    functionName: string | undefined,
  ): (...args: unknown[]) => Promise<unknown>;

  export function createTemporaryReferenceSet(
    ...args: unknown[]
  ): TemporaryReferenceSet;

  export function encodeReply(
    value: unknown,
    options?: {
      temporaryReferences?: TemporaryReferenceSet;
      signal?: AbortSignal;
    },
  ): Promise<string | FormData>;
}

declare module 'react-server-dom-rspack/client.browser' {
  import {
    createTemporaryReferenceSet,
    encodeReply,
    type CallServerCallback,
    type FindSourceMapURLCallback,
    type TemporaryReferenceSet,
  } from 'react-server-dom-rspack/client.edge';

  export { createTemporaryReferenceSet, encodeReply };

  export interface Options {
    callServer?: CallServerCallback;
    environmentName?: string;
    // It's optional but we want to avoid accidentally omitting it.
    findSourceMapURL: FindSourceMapURLCallback | undefined;
    replayConsoleLogs?: boolean;
    temporaryReferences?: TemporaryReferenceSet;
    debugChannel?: { readable?: ReadableStream; writable?: WritableStream };
  }

  export function createFromFetch<T>(
    promiseForResponse: Promise<Response>,
    options?: Options,
  ): Promise<T>;

  export function createFromReadableStream<T>(
    stream: ReadableStream,
    options?: Options,
  ): Promise<T>;
}

declare module 'react-server-dom-rspack/server.edge' {
  export type ImportManifestEntry = {
    id: string | number;
    // chunks is a double indexed array of chunkId / chunkFilename pairs
    chunks: ReadonlyArray<string>;
    name: string;
    async?: boolean;
  };

  export type ClientManifest = {
    [id: string]: ImportManifestEntry;
  };

  export type ServerManifest = {
    [id: string]: ImportManifestEntry;
  };

  export type TemporaryReferenceSet = WeakMap<any, string>;

  export function renderToReadableStream(
    model: any,
    webpackMap: ClientManifest,
    options?: {
      temporaryReferences?: TemporaryReferenceSet;
      environmentName?: string | (() => string);
      // This is actually optional.
      // But we want to not miss callsites accidentally and explicitly choose
      // at each callsite which implementation to choose.
      filterStackFrame:
        | ((
            url: string,
            functionName: string,
            lineNumber: number,
            columnNumber: number,
          ) => boolean)
        | undefined;
      onError?: (error: unknown) => void;
      signal?: AbortSignal;
      debugChannel?: { readable?: ReadableStream; writable?: WritableStream };
    },
  ): ReadableStream<Uint8Array>;

  export function createTemporaryReferenceSet(
    ...args: unknown[]
  ): TemporaryReferenceSet;

  export function decodeReply<T>(
    body: string | FormData,
    webpackMap: ServerManifest,
    options?: {
      temporaryReferences?: TemporaryReferenceSet;
    },
  ): Promise<T>;
  export function decodeReplyFromAsyncIterable<T>(
    iterable: AsyncIterable<[string, string | File]>,
    webpackMap: ServerManifest,
    options?: {
      temporaryReferences?: TemporaryReferenceSet;
    },
  ): Promise<T>;
  export function decodeAction<T>(
    body: FormData,
    serverManifest: ServerManifest,
  ): Promise<() => T> | null;
  export function decodeFormState<S>(
    actionResult: S,
    body: FormData,
    serverManifest: ServerManifest,
  ): Promise<unknown | null>;

  export function registerServerReference<T>(
    reference: T,
    id: string,
    exportName: string | null,
  ): unknown;

  export function createClientModuleProxy(moduleId: string): unknown;
}

declare module 'react-server-dom-rspack/server' {
  export * from 'react-server-dom-rspack/server.node';
}

declare module 'react-server-dom-rspack/server.node' {
  import type { Busboy } from 'busboy';

  export {
    createClientModuleProxy,
    decodeReplyFromAsyncIterable,
    registerServerReference,
    renderToReadableStream,
  } from 'react-server-dom-rspack/server.edge';

  export type TemporaryReferenceSet = WeakMap<any, string>;

  export type ImportManifestEntry = {
    id: string;
    // chunks is a double indexed array of chunkId / chunkFilename pairs
    chunks: Array<string>;
    name: string;
    async?: boolean;
  };

  export type ServerManifest = {
    [id: string]: ImportManifestEntry;
  };

  export type ReactFormState = [
    unknown /* actual state value */,
    string /* key path */,
    string /* Server Reference ID */,
    number /* number of bound arguments */,
  ];

  export function createTemporaryReferenceSet(
    ...args: unknown[]
  ): TemporaryReferenceSet;

  export function decodeReplyFromBusboy(
    busboyStream: Busboy,
    webpackMap: ServerManifest,
    options?: { temporaryReferences?: TemporaryReferenceSet },
  ): Promise<unknown[]>;

  export function decodeReply<T>(
    body: string | FormData,
    webpackMap: ServerManifest,
    options?: { temporaryReferences?: TemporaryReferenceSet },
  ): Promise<T[]>;

  export function decodeAction(
    body: FormData,
    serverManifest: ServerManifest,
  ): Promise<() => unknown> | null;

  export function decodeFormState(
    actionResult: unknown,
    body: FormData,
    serverManifest: ServerManifest,
  ): Promise<ReactFormState | null>;
}
declare module 'react-server-dom-rspack/static' {
  export type TemporaryReferenceSet = WeakMap<any, string>;

  export function prerender(
    children: any,
    webpackMap: {
      readonly [id: string]: {
        readonly id: string | number;
        readonly chunks: readonly string[];
        readonly name: string;
        readonly async?: boolean;
      };
    },
    options?: {
      environmentName?: string | (() => string);
      // This is actually optional.
      // But we want to not miss callsites accidentally and explicitly choose
      // at each callsite which implementation to choose.
      filterStackFrame:
        | ((
            url: string,
            functionName: string,
            lineNumber: number,
            columnNumber: number,
          ) => boolean)
        | undefined;
      identifierPrefix?: string;
      signal?: AbortSignal;
      temporaryReferences?: TemporaryReferenceSet;
      onError?: (error: unknown) => void;
    },
  ): Promise<{
    prelude: ReadableStream<Uint8Array>;
  }>;
}
declare module 'react-server-dom-rspack/client.edge' {
  export interface Options {
    callServer?: CallServerCallback;
    serverConsumerManifest: ServerConsumerManifest;
    nonce?: string;
    encodeFormAction?: EncodeFormActionCallback;
    temporaryReferences?: TemporaryReferenceSet;
    // It's optional but we want to avoid accidentally omitting it.
    findSourceMapURL: FindSourceMapURLCallback | undefined;
    replayConsoleLogs?: boolean;
    environmentName?: string;
    debugChannel?: { readable?: ReadableStream };
  }

  export type EncodeFormActionCallback = <A>(
    id: any,
    args: Promise<A>,
  ) => ReactCustomFormAction;

  export type ReactCustomFormAction = {
    name?: string;
    action?: string;
    encType?: string;
    method?: string;
    target?: string;
    data?: null | FormData;
  };

  export type ImportManifestEntry = {
    id: string | number;
    // chunks is a double indexed array of chunkId / chunkFilename pairs
    chunks: ReadonlyArray<string>;
    name: string;
    async?: boolean;
  };

  export type ServerManifest = {
    [id: string]: ImportManifestEntry;
  };

  export interface ServerConsumerManifest {
    moduleMap: ServerConsumerModuleMap;
    moduleLoading: ModuleLoading | null;
    serverModuleMap: null | ServerManifest;
  }

  export interface ServerConsumerModuleMap {
    [clientId: string]: {
      [clientExportName: string]: ImportManifestEntry;
    };
  }

  export interface ModuleLoading {
    prefix: string;
    crossOrigin?: 'use-credentials' | '';
  }

  type TemporaryReferenceSet = Map<string, unknown>;

  export type CallServerCallback = (
    id: string,
    args: unknown[],
  ) => Promise<unknown>;

  export type FindSourceMapURLCallback = (
    fileName: string,
    environmentName: string,
  ) => null | string;

  export function createFromFetch<T>(
    promiseForResponse: Promise<Response>,
    options?: Options,
  ): Promise<T>;

  export function createFromReadableStream<T>(
    stream: ReadableStream,
    options?: Options,
  ): Promise<T>;

  export function createServerReference(
    id: string,
    callServer: CallServerCallback,
  ): (...args: unknown[]) => Promise<unknown>;

  export function createTemporaryReferenceSet(
    ...args: unknown[]
  ): TemporaryReferenceSet;

  export function encodeReply(
    value: unknown,
    options?: {
      temporaryReferences?: TemporaryReferenceSet;
      signal?: AbortSignal;
    },
  ): Promise<string | FormData>;
}
