/**
 * The following code is modified based on
 * https://github.com/webpack/webpack-dev-server/blob/6045b1e9d63078fb24cac52eb361b7356944cddd/types/lib/Server.d.ts
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack-dev-server/blob/master/LICENSE
 */
import type { ReadStream } from 'node:fs';
import type { IncomingMessage, ServerResponse } from 'node:http';
import type { ServerOptions } from 'node:https';
import type { Server as ConnectApplication } from 'connect-next';
import type {
  Filter as ProxyFilter,
  Options as ProxyOptions,
} from 'http-proxy-middleware';
import type { Options as OpenOptions } from 'open';
import type {
  Compiler,
  Configuration,
  LiteralUnion,
  MultiCompiler,
  MultiStats,
  Stats,
  Watching,
} from '..';

type Logger = ReturnType<Compiler['getInfrastructureLogger']>;
type MultiWatching = MultiCompiler['watch'];

export type DevServerHost = LiteralUnion<
  'local-ip' | 'local-ipv4' | 'local-ipv6',
  string
>;

type BasicServer = import('node:net').Server | import('node:tls').Server;

export type DevServerOpenOptions = OpenOptions;

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

export type DevServerHeaders =
  | {
      key: string;
      value: string;
    }[]
  | Record<string, string | string[]>;

type OutputFileSystem = import('..').OutputFileSystem & {
  statSync: import('fs').StatSyncFn;
  readFileSync: typeof import('fs').readFileSync;
};

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
  readonly disableDotRule?: true;
  readonly htmlAcceptHeaders?: readonly string[];
  readonly index?: string;
  readonly logger?: typeof console.log;
  readonly rewrites?: readonly Rewrite[];
  readonly verbose?: boolean;
};

type DevMiddlewareOptions<
  RequestInternal extends IncomingMessage = IncomingMessage,
  ResponseInternal extends ServerResponse = ServerResponse,
> = {
  mimeTypes?: {
    [key: string]: string;
  };
  mimeTypeDefault?: string;
  writeToDisk?: boolean | ((targetPath: string) => boolean);
  methods?: string[];
  headers?: any;
  publicPath?: NonNullable<Configuration['output']>['publicPath'];
  stats?: Configuration['stats'];
  serverSideRender?: boolean;
  outputFileSystem?: OutputFileSystem;
  index?: string | boolean;
  modifyResponseData?: ModifyResponseData<RequestInternal, ResponseInternal>;
  etag?: 'strong' | 'weak';
  lastModified?: boolean;
  cacheControl?:
    | string
    | number
    | boolean
    | {
        maxAge?: number;
        immutable?: boolean;
      };
  cacheImmutable?: boolean;
};

type BasicApplication = any;
type ChokidarWatchOptions = { [key: string]: any };
type ServeStaticOptions = { [key: string]: any };

type WatchFiles = {
  paths: string | string[];
  options?: ChokidarWatchOptions & {
    aggregateTimeout?: number;
    poll?: number | boolean;
  };
};

export type DevServerStaticItem = {
  directory?: string;
  publicPath?: string | string[];
  staticOptions?: ServeStaticOptions;
  watch?: boolean | NonNullable<WatchFiles['options']>;
};

export type DevServerStatic =
  | string
  | boolean
  | DevServerStaticItem
  | (string | DevServerStaticItem)[];

type ServerType<A extends BasicApplication, S extends BasicServer> =
  | LiteralUnion<'http' | 'https' | 'http2', string>
  | ((arg0: ServerOptions, arg1: A) => S);

type ServerConfiguration<A extends BasicApplication, S extends BasicServer> = {
  type?: ServerType<A, S>;
  options?: ServerOptions;
};

type WebSocketServerConfiguration = {
  type?: string | Function;
  options?: Record<string, any>;
};

type NextFunction = (err?: any) => void;

export type DevServerProxyConfigArrayItem = {
  /**
   * Alias for `pathFilter` in `http-proxy-middleware` options.
   * When both `context` and `pathFilter` are provided, `pathFilter` takes precedence.
   */
  context?: ProxyFilter;
} & ProxyOptions;

export type DevServerProxyConfigArray = (
  | DevServerProxyConfigArrayItem
  | ((
      req?: IncomingMessage,
      res?: ServerResponse,
      next?: NextFunction,
    ) => DevServerProxyConfigArrayItem)
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

export type DevServerMiddlewareHandler<
  RequestInternal extends IncomingMessage = IncomingMessage,
  ResponseInternal extends ServerResponse = ServerResponse,
> = (
  req: RequestInternal,
  res: ResponseInternal,
  next: NextFunction,
) => void | Promise<void>;

type DevServerMiddlewareObject<
  RequestInternal extends IncomingMessage = IncomingMessage,
  ResponseInternal extends ServerResponse = ServerResponse,
> = {
  name?: string;
  path?: string;
  middleware: DevServerMiddlewareHandler<RequestInternal, ResponseInternal>;
};

export type DevServerMiddleware =
  | DevServerMiddlewareObject
  | DevServerMiddlewareHandler;

type OverlayMessageOptions = boolean | ((error: Error) => void);

export type DevServerWebSocketURL = {
  hostname?: string;
  password?: string;
  pathname?: string;
  port?: string | number;
  protocol?: string;
  username?: string;
};

export type DevServerClient = {
  logging?: 'none' | 'error' | 'warn' | 'info' | 'log' | 'verbose';
  overlay?:
    | boolean
    | {
        warnings?: OverlayMessageOptions;
        errors?: OverlayMessageOptions;
        runtimeErrors?: OverlayMessageOptions;
      };
  progress?: boolean;
  reconnect?: number | boolean;
  webSocketTransport?: LiteralUnion<'ws', string>;
  webSocketURL?: string | DevServerWebSocketURL;
};

export type DevServerOptions<
  A extends BasicApplication = ConnectApplication,
  S extends BasicServer = BasicServer,
> = {
  ipc?: string | boolean;
  host?: DevServerHost;
  port?: Port;
  hot?: boolean | 'only';
  liveReload?: boolean;
  devMiddleware?: DevMiddlewareOptions;
  compress?: boolean;
  allowedHosts?: LiteralUnion<'auto' | 'all', string> | string[];
  historyApiFallback?: boolean | HistoryApiFallbackOptions;
  watchFiles?: string | string[] | WatchFiles | (string | WatchFiles)[];
  static?: DevServerStatic;
  server?: ServerType<A, S> | ServerConfiguration<A, S>;
  app?: () => Promise<A>;
  webSocketServer?:
    | boolean
    | LiteralUnion<'ws', string>
    | WebSocketServerConfiguration;
  proxy?: DevServerProxyConfigArray;
  open?: string | boolean | OpenOptions | (string | OpenOptions)[];
  setupExitSignals?: boolean;
  client?: boolean | DevServerClient;
  headers?:
    | DevServerHeaders
    | ((
        req: IncomingMessage,
        res: ServerResponse,
        context: DevMiddlewareContext | undefined,
      ) => DevServerHeaders);
  onListening?: (devServer: any) => void;
  setupMiddlewares?: (
    middlewares: DevServerMiddleware[],
    devServer: any,
  ) => DevServerMiddleware[];
};
