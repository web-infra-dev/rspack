import fs from 'fs-extra';
import path from 'path';
import { DevServer } from './server';
import chokidar from 'chokidar';
import { Rspack } from './rspack';
class MockBundler {
  /**
   * first build
   */
  async build() {}
  /**
   * rebuild and send back updateInfo
   * @param changeFile
   */
  async rebuild(changeFile: string[]) {}
}
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
};
export async function run(options: BundlerOptions) {
  const { root } = options;
  const entry = path.resolve(root, 'index.js');
  const watcher = chokidar.watch(root);

  const bundler = new Rspack({
    entries: [entry],
    minify: false,
    entryFileNames: 'main.js',
  });
  const server = new DevServer({
    root,
    public: 'dist',
  });
  await bundler.build();
  watcher.on('change', (path) => {
    console.log('change:', path);
    /**
     * @todo update logic
     * 目前会重新触发自该模块开始的全量编译，webpack也是这么做吗
     */
    const update = bundler.build();
    server.broadcast({
      type: 'js-update',
      path: path,
      timestamp: Date.now(),
      update: update,
    });
  });
  const htmlPath = path.resolve(__dirname, '../client/index.html');
  fs.ensureDirSync(path.resolve(root, 'dist'));
  fs.copyFileSync(htmlPath, path.resolve(root, 'dist/index.html'));
  server.start();
}
