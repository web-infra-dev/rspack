/**
 * The following code is modified based on
 * https://github.com/webpack/webpack-dev-server/blob/6045b1e9d63078fb24cac52eb361b7356944cddd/types/lib/Server.d.ts
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack-dev-server/blob/master/LICENSE
 */

import type * as http from 'node:http';
import type * as net from 'node:net';
import type * as stream from 'node:stream';
import type * as url from 'node:url';
import type {
  Compiler,
  LiteralUnion,
  MultiCompiler,
  MultiStats,
  Stats,
  Watching,
} from '..';

type Logger = ReturnType<Compiler['getInfrastructureLogger']>;
type MultiWatching = MultiCompiler['watch'];
type BasicServer = import('net').Server | import('tls').Server;

type ReadStream = import('fs').ReadStream;
type IncomingMessage = import('http').IncomingMessage;
type ServerResponse = import('http').ServerResponse;
type ServerOptions = import('https').ServerOptions & {
  spdy?: {
    plain?: boolean | undefined;
    ssl?: boolean | undefined;
    'x-forwarded-for'?: string | undefined;
    protocol?: string | undefined;
    protocols?: string[] | undefined;
  };
};

type ResponseData = {
  data: Buffer | ReadStream;
  byteLength: number;
};
type ModifyResponseData<
  RequestInternal extends IncomingMessage = IncomingMessage,
  ResponseInternal extends ServerResponse = ServerResponse,
> = (
  req: RequestInternal,
  res: ResponseInternal,
  data: Buffer | ReadStream,
  byteLength: number,
) => ResponseData;
type Headers =
  | {
      key: string;
      value: string;
    }[]
  | Record<string, string | string[]>;
type OutputFileSystem = import('..').OutputFileSystem & {
  statSync: import('fs').StatSyncFn;
  readFileSync: typeof import('fs').readFileSync;
};
type RspackConfiguration = import('..').Configuration;
type Port = number | LiteralUnion<'auto', string>;

type HistoryContext = {
  readonly match: RegExpMatchArray;
  readonly parsedUrl: import('url').Url;
  readonly request: any;
};
type RewriteTo = (context: HistoryContext) => string;
type Rewrite = {
  readonly from: RegExp;
  readonly to: string | RegExp | RewriteTo;
};
type HistoryApiFallbackOptions = {
  readonly disableDotRule?: true | undefined;
  readonly htmlAcceptHeaders?: readonly string[] | undefined;
  readonly index?: string | undefined;
  readonly logger?: typeof console.log | undefined;
  readonly rewrites?: readonly Rewrite[] | undefined;
  readonly verbose?: boolean | undefined;
};

type DevMiddlewareOptions<
  RequestInternal extends IncomingMessage = IncomingMessage,
  ResponseInternal extends ServerResponse = ServerResponse,
> = {
  mimeTypes?:
    | {
        [key: string]: string;
      }
    | undefined;
  mimeTypeDefault?: string | undefined;
  writeToDisk?: boolean | ((targetPath: string) => boolean) | undefined;
  methods?: string[] | undefined;
  headers?: any;
  publicPath?: NonNullable<RspackConfiguration['output']>['publicPath'];
  stats?: RspackConfiguration['stats'];
  serverSideRender?: boolean | undefined;
  outputFileSystem?: OutputFileSystem | undefined;
  index?: string | boolean | undefined;
  modifyResponseData?:
    | ModifyResponseData<RequestInternal, ResponseInternal>
    | undefined;
  etag?: 'strong' | 'weak' | undefined;
  lastModified?: boolean | undefined;
  cacheControl?:
    | string
    | number
    | boolean
    | {
        maxAge?: number;
        immutable?: boolean;
      }
    | undefined;
  cacheImmutable?: boolean | undefined;
};

type BasicApplication = any;
type BonjourServer = Record<string, any>;
type ChokidarWatchOptions = { [key: string]: any };
type ServeIndexOptions = { [key: string]: any };
type ServeStaticOptions = { [key: string]: any };
type HttpProxyMiddlewareOptionsFilter = any;
type Request = IncomingMessage;
type Response = ServerResponse;

type WatchFiles = {
  paths: string | string[];
  options?:
    | (ChokidarWatchOptions & {
        aggregateTimeout?: number;
        ignored?: ChokidarWatchOptions['ignored'];
        poll?: number | boolean;
      })
    | undefined;
};

type Static = {
  directory?: string | undefined;
  publicPath?: string | string[] | undefined;
  serveIndex?: boolean | ServeIndexOptions | undefined;
  staticOptions?: ServeStaticOptions | undefined;
  watch?:
    | boolean
    | (ChokidarWatchOptions & {
        aggregateTimeout?: number;
        ignored?: ChokidarWatchOptions['ignored'];
        poll?: number | boolean;
      })
    | undefined;
};

type ServerType<
  A extends BasicApplication = BasicApplication,
  S extends BasicServer = import('http').Server<
    typeof import('http').IncomingMessage,
    typeof import('http').ServerResponse
  >,
> =
  | LiteralUnion<'http' | 'https' | 'spdy' | 'http2', string>
  | ((arg0: ServerOptions, arg1: A) => S);

type ServerConfiguration<
  A extends BasicApplication = BasicApplication,
  S extends BasicServer = import('http').Server<
    typeof import('http').IncomingMessage,
    typeof import('http').ServerResponse
  >,
> = {
  type?: ServerType<A, S> | undefined;
  options?: ServerOptions | undefined;
};

type WebSocketServerConfiguration = {
  type?: string | Function | undefined;
  options?: Record<string, any> | undefined;
};
type NextFunction = (err?: any) => void;
type ProxyConfigArrayItem = {
  path?: HttpProxyMiddlewareOptionsFilter;
  context?: HttpProxyMiddlewareOptionsFilter;
} & {
  bypass?: ByPass;
} & HttpProxyMiddlewareOptions;
type ByPass = (
  req: Request,
  res: Response,
  proxyConfig: ProxyConfigArrayItem,
) => any;
type ProxyConfigArray = (
  | ProxyConfigArrayItem
  | ((
      req?: Request,
      res?: Response,
      next?: NextFunction,
    ) => ProxyConfigArrayItem)
)[];
type Callback = (stats?: Stats | MultiStats) => any;
type DevMiddlewareContext<
  _RequestInternal extends IncomingMessage = IncomingMessage,
  _ResponseInternal extends ServerResponse = ServerResponse,
> = {
  state: boolean;
  stats: Stats | MultiStats | undefined;
  callbacks: Callback[];
  options: any;
  compiler: Compiler | MultiCompiler;
  watching: Watching | MultiWatching | undefined;
  logger: Logger;
  outputFileSystem: OutputFileSystem;
};
type Server = any;

export type MiddlewareHandler<
  RequestInternal extends Request = Request,
  ResponseInternal extends Response = Response,
> = (
  req: RequestInternal,
  res: ResponseInternal,
  next: NextFunction,
) => void | Promise<void>;

type MiddlewareObject<
  RequestInternal extends Request = Request,
  ResponseInternal extends Response = Response,
> = {
  name?: string;
  path?: string;
  middleware: MiddlewareHandler<RequestInternal, ResponseInternal>;
};
export type Middleware<
  RequestInternal extends Request = Request,
  ResponseInternal extends Response = Response,
> =
  | MiddlewareObject<RequestInternal, ResponseInternal>
  | MiddlewareHandler<RequestInternal, ResponseInternal>;

type OpenApp = {
  name?: string | undefined;
  arguments?: string[] | undefined;
};
type Open = {
  app?: string | string[] | OpenApp | undefined;
  target?: string | string[] | undefined;
};
type OverlayMessageOptions = boolean | ((error: Error) => void);
type WebSocketURL = {
  hostname?: string | undefined;
  password?: string | undefined;
  pathname?: string | undefined;
  port?: string | number | undefined;
  protocol?: string | undefined;
  username?: string | undefined;
};
type ClientConfiguration = {
  logging?: 'none' | 'error' | 'warn' | 'info' | 'log' | 'verbose' | undefined;
  overlay?:
    | boolean
    | {
        warnings?: OverlayMessageOptions;
        errors?: OverlayMessageOptions;
        runtimeErrors?: OverlayMessageOptions;
      }
    | undefined;
  progress?: boolean | undefined;
  reconnect?: number | boolean | undefined;
  webSocketTransport?: string | undefined;
  webSocketURL?: string | WebSocketURL | undefined;
};

export type DevServerOptions<
  A extends BasicApplication = BasicApplication,
  S extends BasicServer = import('http').Server<
    typeof import('http').IncomingMessage,
    typeof import('http').ServerResponse
  >,
> = {
  ipc?: string | boolean | undefined;
  host?: string | undefined;
  port?: Port | undefined;
  hot?: boolean | 'only' | undefined;
  liveReload?: boolean | undefined;
  devMiddleware?: DevMiddlewareOptions | undefined;
  compress?: boolean | undefined;
  allowedHosts?: string | string[] | undefined;
  historyApiFallback?: boolean | HistoryApiFallbackOptions | undefined;
  bonjour?: boolean | BonjourServer | undefined;
  watchFiles?:
    | string
    | string[]
    | WatchFiles
    | (string | WatchFiles)[]
    | undefined;
  static?: string | boolean | Static | (string | Static)[] | undefined;
  server?: ServerType<A, S> | ServerConfiguration<A, S> | undefined;
  app?: (() => Promise<A>) | undefined;
  webSocketServer?: string | boolean | WebSocketServerConfiguration | undefined;
  proxy?: ProxyConfigArray | undefined;
  open?: string | boolean | Open | (string | Open)[] | undefined;
  setupExitSignals?: boolean | undefined;
  client?: boolean | ClientConfiguration | undefined;
  headers?:
    | Headers
    | ((
        req: Request,
        res: Response,
        context: DevMiddlewareContext | undefined,
      ) => Headers)
    | undefined;
  onListening?: ((devServer: Server) => void) | undefined;
  setupMiddlewares?:
    | ((middlewares: Middleware[], devServer: Server) => Middleware[])
    | undefined;
};

interface HttpProxyMiddlewareOptions extends HttpProxyServerOptions {
  pathRewrite?:
    | {
        [regexp: string]: string;
      }
    | ((path: string, req: Request) => string)
    | ((path: string, req: Request) => Promise<string>);
  router?:
    | {
        [hostOrPath: string]: HttpProxyServerOptions['target'];
      }
    | ((req: Request) => HttpProxyServerOptions['target'])
    | ((req: Request) => Promise<HttpProxyServerOptions['target']>);
  logLevel?: 'debug' | 'info' | 'warn' | 'error' | 'silent';
  logProvider?: LogProviderCallback;
  onError?: OnErrorCallback;
  onProxyRes?: OnProxyResCallback;
  onProxyReq?: OnProxyReqCallback;
  onProxyReqWs?: OnProxyReqWsCallback;
  onOpen?: OnOpenCallback;
  onClose?: OnCloseCallback;
}

interface LogProvider {
  log: Logger;
  debug?: Logger;
  info?: Logger;
  warn?: Logger;
  error?: Logger;
}

type LogProviderCallback = (provider: LogProvider) => LogProvider;
type OnErrorCallback = (
  err: Error,
  req: Request,
  res: Response,
  target?: string | Partial<url.Url>,
) => void;
type OnProxyResCallback = (
  proxyRes: http.IncomingMessage,
  req: Request,
  res: Response,
) => void;
type OnProxyReqCallback = (
  proxyReq: http.ClientRequest,
  req: Request,
  res: Response,
  options: HttpProxyServerOptions,
) => void;
type OnProxyReqWsCallback = (
  proxyReq: http.ClientRequest,
  req: Request,
  socket: net.Socket,
  options: HttpProxyServerOptions,
  head: any,
) => void;
type OnCloseCallback = (
  proxyRes: Response,
  proxySocket: net.Socket,
  proxyHead: any,
) => void;
type OnOpenCallback = (proxySocket: net.Socket) => void;
interface HttpProxyServerOptions {
  /** URL string to be parsed with the url module. */
  target?: HttpProxyTarget | undefined;
  /** URL string to be parsed with the url module. */
  forward?: HttpProxyTargetUrl | undefined;
  /** Object to be passed to http(s).request. */
  agent?: any;
  /** Object to be passed to https.createServer(). */
  ssl?: any;
  /** If you want to proxy websockets. */
  ws?: boolean | undefined;
  /** Adds x- forward headers. */
  xfwd?: boolean | undefined;
  /** Verify SSL certificate. */
  secure?: boolean | undefined;
  /** Explicitly specify if we are proxying to another proxy. */
  toProxy?: boolean | undefined;
  /** Specify whether you want to prepend the target's path to the proxy path. */
  prependPath?: boolean | undefined;
  /** Specify whether you want to ignore the proxy path of the incoming request. */
  ignorePath?: boolean | undefined;
  /** Local interface string to bind for outgoing connections. */
  localAddress?: string | undefined;
  /** Changes the origin of the host header to the target URL. */
  changeOrigin?: boolean | undefined;
  /** specify whether you want to keep letter case of response header key */
  preserveHeaderKeyCase?: boolean | undefined;
  /** Basic authentication i.e. 'user:password' to compute an Authorization header. */
  auth?: string | undefined;
  /** Rewrites the location hostname on (301 / 302 / 307 / 308) redirects, Default: null. */
  hostRewrite?: string | undefined;
  /** Rewrites the location host/ port on (301 / 302 / 307 / 308) redirects based on requested host/ port.Default: false. */
  autoRewrite?: boolean | undefined;
  /** Rewrites the location protocol on (301 / 302 / 307 / 308) redirects to 'http' or 'https'.Default: null. */
  protocolRewrite?: string | undefined;
  /** rewrites domain of set-cookie headers. */
  cookieDomainRewrite?:
    | false
    | string
    | { [oldDomain: string]: string }
    | undefined;
  /** rewrites path of set-cookie headers. Default: false */
  cookiePathRewrite?:
    | false
    | string
    | { [oldPath: string]: string }
    | undefined;
  /** object with extra headers to be added to target requests. */
  headers?: { [header: string]: string } | undefined;
  /** Timeout (in milliseconds) when proxy receives no response from target. Default: 120000 (2 minutes) */
  proxyTimeout?: number | undefined;
  /** Timeout (in milliseconds) for incoming requests */
  timeout?: number | undefined;
  /** Specify whether you want to follow redirects. Default: false */
  followRedirects?: boolean | undefined;
  /** If set to true, none of the webOutgoing passes are called and it's your responsibility to appropriately return the response by listening and acting on the proxyRes event */
  selfHandleResponse?: boolean | undefined;
  /** Buffer */
  buffer?: stream.Stream | undefined;
  /** Explicitly set the method type of the ProxyReq */
  method?: string | undefined;
}

interface HttpProxyTargetDetailed {
  host: string;
  port: number;
  protocol?: string | undefined;
  hostname?: string | undefined;
  socketPath?: string | undefined;
  key?: string | undefined;
  passphrase?: string | undefined;
  pfx?: Buffer | string | undefined;
  cert?: string | undefined;
  ca?: string | undefined;
  ciphers?: string | undefined;
  secureProtocol?: string | undefined;
}

type HttpProxyTarget = HttpProxyTargetUrl | HttpProxyTargetDetailed;
type HttpProxyTargetUrl = string | Partial<url.Url>;
