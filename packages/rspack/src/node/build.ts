import fs from 'fs-extra';
import path from 'path';
import { DevServer } from './server';
import chokidar from 'chokidar';
import { Rspack } from './rspack';
import { RawOptions } from '@rspack/binding';
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
export type BundlerOptions = Partial<RawOptions> & {
  entry: Record<string, string>;
  root: string;
  manualChunks: Record<string, string[]>;
  loader?: Record<string, 'dataURI' | 'json'>;
  inlineStyle?: boolean;
  alias?: Record<string, string>;
  react: Record<string, any>;
  sourceMap: boolean;
};

class RspackPluginTest {
  constructor() {

  }
}


export async function run(options: BundlerOptions) {
  const { root, entry, loader, inlineStyle, alias, react } = options;
  // const entry = path.resolve(root, 'index.js');
  const watcher = chokidar.watch(root, {
    ignored: path.resolve(root, 'dist'),
  });

  const bundler = new Rspack({
    root,
    entries: Object.values(entry),
    minify: false,
    entryFileNames: '[name].js',
    outdir: path.resolve(root, 'dist'),
    loader,
    inlineStyle,
    alias,
    refresh: options.react.refresh,
    sourceMap: options.sourceMap,
    codeSplitting: options.codeSplitting,
  });
  const server = new DevServer({
    root,
    public: 'dist',
  });
  await bundler.build();
  watcher.on('change', async (id) => {
    const url = path.relative(root, id);
    console.log('hmr:start', url);
    /**
     * @todo update logic
     * 目前会重新触发自该模块开始的全量编译，webpack也是这么做吗
     */
    const update = await bundler.rebuild(id);
    const sourceUrl = `\n//# sourceURL=${path}`;
    server.broadcast({
      type: 'js-update',
      path: url,
      timestamp: Date.now(),
      code: Object.values(update).join(';\n') + `invalidate(${JSON.stringify(url)})` + sourceUrl,
    });
    console.log('hmr:end', url);
  });
  const htmlPath = path.resolve(__dirname, '../client/index.html');
  fs.ensureDirSync(path.resolve(root, 'dist'));
  fs.copyFileSync(htmlPath, path.resolve(root, 'dist/index.html'));
  server.start();
}
