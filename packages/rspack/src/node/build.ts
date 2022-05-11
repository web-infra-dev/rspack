import fs from 'fs-extra';
import path from 'path';
import { DevServer } from './server';
import chokidar from 'chokidar';
import { Rspack } from './rspack';
type Defer = {
  resolve: any;
  reject: any;
  promise: any;
};
const Defer = (): Defer => {
  const deferred = {} as Defer;

  deferred.promise = new Promise((resolve, reject) => {
    deferred.resolve = resolve;
    deferred.reject = reject;
  });

  return deferred;
};
export type BundlerOptions = {
  entry: Record<string, string>;
  root: string;
  manualChunks: Record<string, string[]>;
  loader?: Record<string, 'dataURI' | 'json'>;
  inlineStyle?: boolean;
  alias?: Record<string, string>;
  react: Record<string, any>;
};
export async function run(options: BundlerOptions) {
  const { root, entry, loader, inlineStyle, alias, react } = options;
  // const entry = path.resolve(root, 'index.js');
  const watcher = chokidar.watch(root, {
    ignored: path.resolve(root, 'dist'),
  });

  const bundler = new Rspack({
    entries: Object.values(entry),
    minify: false,
    entryFileNames: '[name].js',
    outdir: path.resolve(root, 'dist'),
    loader,
    inlineStyle,
    alias,
    refresh: options.react.refresh,
  });
  const server = new DevServer({
    root,
    public: 'dist',
  });
  await bundler.build();
  watcher.on('change', async (path) => {
    console.log('change:', path);
    /**
     * @todo update logic
     * 目前会重新触发自该模块开始的全量编译，webpack也是这么做吗
     */
    const update = await bundler.rebuild(path);
    const sourceUrl = `\n//# sourceURL=${path}`;
    server.broadcast({
      type: 'js-update',
      path: path,
      timestamp: Date.now(),
      code: Object.values(update).join(';\n') + `invalidate(${JSON.stringify(path)})` + sourceUrl,
    });
  });
  const htmlPath = path.resolve(__dirname, '../client/index.html');
  fs.ensureDirSync(path.resolve(root, 'dist'));
  fs.copyFileSync(htmlPath, path.resolve(root, 'dist/index.html'));
  server.start();
}
