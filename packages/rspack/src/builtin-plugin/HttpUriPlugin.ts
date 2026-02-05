import type { IncomingMessage } from 'node:http';
import { createRequire } from 'node:module';
import path from 'node:path';
import {
  type BuiltinPlugin,
  BuiltinPluginName,
  type RawHttpUriPluginOptions,
} from '@rspack/binding';
import type { Compiler } from '../Compiler';
import { memoize } from '../util/memoize';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

const require = createRequire(import.meta.url);

export type HttpUriPluginOptionsAllowedUris = (string | RegExp)[];

export type HttpUriPluginOptions = {
  /**
   * A list of allowed URIs
   */
  allowedUris: HttpUriPluginOptionsAllowedUris;
  /**
   * Define the location to store the lockfile
   */
  lockfileLocation?: string;
  /**
   * Define the location for caching remote resources
   */
  cacheLocation?: string | false;
  /**
   * Detect changes to remote resources and upgrade them automatically
   */
  upgrade?: boolean;
  // /**
  //  * Specify the proxy server to use for fetching remote resources
  //  */
  // proxy?: string;
  // /**
  //  * Freeze the remote resources and lockfile. Any modification to the lockfile or resource contents will result in an error
  //  */
  // frozen?: boolean;
  /**
   * Custom http client
   */
  httpClient?: RawHttpUriPluginOptions['httpClient'];
};

const getHttp = memoize(() => require('node:http'));
const getHttps = memoize(() => require('node:https'));

function compatibleFetch(
  url: string,
  options: { headers: Record<string, string> },
) {
  const parsedURL = new URL(url);
  const send: typeof import('node:http') =
    parsedURL.protocol === 'https:' ? getHttps() : getHttp();
  const {
    createBrotliDecompress,
    createGunzip,
    createInflate,
  } = require('node:zlib');

  return new Promise<{ res: IncomingMessage; body: Buffer }>(
    (resolve, reject) => {
      console.log(`[HTTP-CLIENT-FETCH] Starting request: ${url}`);
      const req = send.get(url, options, (res) => {
        console.log(
          `[HTTP-CLIENT-FETCH] Response received: ${url}, status: ${res.statusCode}`,
        );
        // align with https://github.com/webpack/webpack/blob/dec18718be5dfba28f067fb3827dd620a1f33667/lib/schemes/HttpUriPlugin.js#L807
        const contentEncoding = res.headers['content-encoding'];
        /** @type {Readable} */
        let stream = res;
        if (contentEncoding === 'gzip') {
          stream = stream.pipe(createGunzip()) as IncomingMessage;
        } else if (contentEncoding === 'br') {
          stream = stream.pipe(createBrotliDecompress()) as IncomingMessage;
        } else if (contentEncoding === 'deflate') {
          stream = stream.pipe(createInflate()) as IncomingMessage;
        }
        const chunks: Buffer[] = [];
        stream.on('data', (chunk) => {
          chunks.push(chunk);
        });
        stream.on('end', () => {
          console.log(
            `[HTTP-CLIENT-FETCH] Stream ended: ${url}, chunks: ${chunks.length}`,
          );
          const bodyBuffer = Buffer.concat(chunks);
          if (!res.complete) {
            console.error(
              `[HTTP-CLIENT-FETCH] Request terminated early: ${url}`,
            );
            reject(new Error(`${url} request was terminated early`));
            return;
          }
          console.log(`[HTTP-CLIENT-FETCH] Request complete: ${url}`);
          resolve({
            res,
            body: bodyBuffer,
          });
        });
        stream.on('error', (e) => {
          console.error(`[HTTP-CLIENT-FETCH] Stream error: ${url}`, e);
          reject(e);
        });
      });

      req.on('error', (e) => {
        console.error(`[HTTP-CLIENT-FETCH] Request error: ${url}`, e);
        reject(e);
      });

      req.on('socket', (socket) => {
        console.log(`[HTTP-CLIENT-FETCH] Socket assigned: ${url}`);
        console.log(
          `[HTTP-CLIENT-FETCH] Socket info: connecting=${socket.connecting}, destroyed=${socket.destroyed}, pending=${socket.pending}`,
        );

        socket.on('connect', () => {
          console.log(`[HTTP-CLIENT-FETCH] Socket connected: ${url}`);
        });

        socket.on('timeout', () => {
          console.error(`[HTTP-CLIENT-FETCH] Socket timeout: ${url}`);
          socket.destroy();
        });

        socket.on('end', () => {
          console.log(`[HTTP-CLIENT-FETCH] Socket ended: ${url}`);
        });

        socket.on('close', (hadError) => {
          console.log(
            `[HTTP-CLIENT-FETCH] Socket closed: ${url}, hadError=${hadError}`,
          );
        });

        socket.on('error', (e) => {
          console.error(`[HTTP-CLIENT-FETCH] Socket error: ${url}`, e);
        });
      });
    },
  );
}

const defaultHttpClientForNode = async (
  url: string,
  headers: Record<string, string>,
) => {
  const startTime = Date.now();
  console.log(`[HTTP-CLIENT] Request start: ${url} at ${startTime}`);

  // Return a promise that resolves to the response
  // setting redirect: "manual" to prevent automatic redirection which will break the redirect logic in rust plugin
  // webpack use require('http').get while rspack use fetch which treats redirect differently
  try {
    const { res, body } = await compatibleFetch(url, { headers });
    const duration = Date.now() - startTime;
    console.log(`[HTTP-CLIENT] Response received: ${url}`);
    console.log(
      `[HTTP-CLIENT]   Status: ${res.statusCode}, Duration: ${duration}ms`,
    );
    console.log(`[HTTP-CLIENT]   Body size: ${body.length} bytes`);
    if (res.statusCode && res.statusCode >= 300 && res.statusCode < 400) {
      console.log(`[HTTP-CLIENT]   Redirect to: ${res.headers.location}`);
    }
    const responseHeaders: Record<string, string> = {};
    for (const [key, value] of Object.entries(res.headers)) {
      if (Array.isArray(value)) {
        responseHeaders[key] = value.join(', ');
      } else {
        responseHeaders[key] = value!;
      }
    }

    console.log(`[HTTP-CLIENT] Request complete: ${url}`);

    // Return the standardized format
    return {
      status: res.statusCode!,
      headers: responseHeaders,
      body: Buffer.from(body),
    };
  } catch (error) {
    const duration = Date.now() - startTime;
    console.error(`[HTTP-CLIENT] Request failed: ${url}`);
    console.error(`[HTTP-CLIENT]   Error: ${error}`);
    console.error(`[HTTP-CLIENT]   Duration: ${duration}ms`);
    throw error;
  }
};

/**
 * Default HTTP client for browser
 * We directly use fetch API in browser since we don't need to worry about the compatibility
 */
const defaultHttpClientForBrowser = async (
  url: string,
  headers: Record<string, string>,
) => {
  const res = await fetch(url, { headers });
  const responseHeaders: Record<string, string> = {};
  for (const [key, value] of Object.entries(res.headers)) {
    if (Array.isArray(value)) {
      responseHeaders[key] = value.join(', ');
    } else {
      responseHeaders[key] = value!;
    }
  }

  return {
    status: res.status,
    headers: responseHeaders,
    body: Buffer.from(await res.arrayBuffer()),
  };
};

/**
 * Plugin that allows loading modules from HTTP URLs
 */
export class HttpUriPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.HttpUriPlugin;
  affectedHooks = 'compilation' as const;

  constructor(private options: HttpUriPluginOptions) {
    super();
  }

  raw(compiler: Compiler): BuiltinPlugin | undefined {
    const options = this.options;
    const lockfileLocation =
      options.lockfileLocation ??
      path.join(
        compiler.context,
        compiler.name ? `${compiler.name}.rspack.lock` : 'rspack.lock',
      );
    const cacheLocation =
      options.cacheLocation === false
        ? undefined
        : (options.cacheLocation ?? `${lockfileLocation}.data`);

    const defaultHttpClient = IS_BROWSER
      ? defaultHttpClientForBrowser
      : defaultHttpClientForNode;

    const raw: RawHttpUriPluginOptions = {
      allowedUris: options.allowedUris,
      lockfileLocation,
      cacheLocation,
      upgrade: options.upgrade ?? false,
      // frozen: options.frozen,
      // proxy: options.proxy,
      httpClient: options.httpClient ?? defaultHttpClient,
    };
    return createBuiltinPlugin(this.name, raw);
  }
}
