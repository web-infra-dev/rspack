import fs from 'fs-extra';
import path from 'path';
import { DevServer } from './server';
import chokidar from 'chokidar';
import { RawOptions } from '@rspack/binding';
import { Rspack, RspackRawOptions } from './rspack';
import { LessPlugin } from './rspack/plugins/less';
import log from 'why-is-node-running';

type Defer = { resolve: any; reject: any; promise: any };
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
  command: 'dev' | 'build';
};

export async function run(options: RspackRawOptions, command: 'dev' | 'build') {
  console.time('build');
  const root = options.root;
  const outdir = path.resolve(options.root, 'dist');

  const sourceMap: RawOptions['output']['sourceMap'] = (() => {
    if (typeof options.output?.sourceMap === 'boolean') {
      if (options.output.sourceMap) {
        return 'inline';
      }
      return 'none';
    }
    return options.output?.sourceMap;
  })();

  const outputConfig = {
    ...options.output,
    outdir,
    sourceMap,
  };

  const bundler = new Rspack({
    ...options,
    output: outputConfig,

    plugins: [LessPlugin({ root: options.root })],
  });
  await bundler.build();
  /**
   * comment out to diagnostics node not exit problem
   */
  // log();
  if (command === 'dev') {
    // const entry = path.resolve(root, 'index.js');
    const watcher = chokidar.watch(root, {
      ignored: path.resolve(root, 'dist'),
    });
    const server = new DevServer({ root, public: 'dist', bundler });
    console.timeEnd('build');
    watcher.on('change', async (id) => {
      let url = path.relative(root, id);
      if (url.startsWith('./') || url.startsWith('../')) {
      } else {
        url = './' + url;
      }
      console.time(`hmr:${url}`);
      /**
       * @todo update logic
       * 目前会重新触发自该模块开始的全量编译，webpack也是这么做吗
       */
      const update = await bundler.rebuild([id]);
      const sourceUrl = `\n//# sourceURL=${path}`;
      server.broadcast({
        type: 'js-update',
        path: url,
        timestamp: Date.now(),
        code: Object.values(update).join(';\n') + `rs.invalidate(${JSON.stringify(url)})` + sourceUrl,
      });
      console.timeEnd(`hmr:${url}`);
    });
    const htmlPath = path.resolve(__dirname, '../client/index.html');
    fs.ensureDirSync(path.resolve(root, 'dist'));
    fs.copyFileSync(htmlPath, path.resolve(root, 'dist/index.html'));
    server.start();
  }
}
