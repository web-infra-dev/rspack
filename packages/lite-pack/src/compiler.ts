import { Loader } from './loader';
import { Resolver } from './resolver';
import { AsyncQueue } from './queue';
import { ModuleNode, NormalModuleOptions } from './module';
import fs from 'fs-extra';
import path from 'path';
import { ModuleGraph } from './module-graph';
import { Bundler } from './bundle';
import { DevServer } from './server';
import chokidar from 'chokidar';
type Defer = {
  resolve:any;
  reject: any;
  promise: any;
}
const Defer = (): Defer => {
	const deferred = {} as Defer;

	deferred.promise = new Promise((resolve, reject) => {
		deferred.resolve = resolve;
		deferred.reject = reject;
	});

	return deferred;
};
export class Compiler {
  options: BundlerOptions;
  entry: Record<string, string>;
  root: string;
  loader: Loader;
  resolver: Resolver;
  buildQueue: AsyncQueue<ModuleNode>;
  moduleGraph: ModuleGraph;
  private constructor(options: BundlerOptions) {
    this.options = options;
    this.entry = options.entry;
    this.loader = new Loader();
    this.resolver = new Resolver();
    this.root = options.root;

    this.buildQueue = new AsyncQueue({
      name: 'build',
      processor: this._buildModule.bind(this),
    });
    this.moduleGraph = new ModuleGraph();
  }
  static create(options: BundlerOptions) {
    return new Compiler(options);
  }
  _buildModule(mod: ModuleNode, done: Function) {
    mod.build();
    done();
  }
  async generate_module_graph() {
    const p = Defer()
    for (const [key,entry] of Object.entries(this.entry)) {
      this.addModule(
        ModuleNode.create({
          path: entry,
          resolveDir: this.root,
          importer: '',
          compiler: this,
          isEntry: true,
          entryKey: key,
          importKind: 'entry-point'
        })
      );
    }
  }
  buildModule(module: ModuleNode) {
    this.buildQueue.add(module, (err?) => {
      if (err) {
        console.error('build module failed', err);
      } else {
        //console.info('build module success:', module.path)
      }
    });
  }
  addModule(module: ModuleNode) {
    this.buildModule(module);
  }
  generate() {
    const bundler = new Bundler(this.moduleGraph, this.options);
    bundler.build();
    return bundler.output;
  }
  async build(){
    await this.generate_module_graph();
    return this.generate();
  }
}

export type BundlerOptions  = {
  entry: Record<string,string>,
  root: string,
  manualChunks: Record<string,string[]>
}
export async function build(options: BundlerOptions) {
  const { root} = options;

  const dstPath = path.resolve(root, 'dist');
  fs.ensureDirSync(dstPath);
  const watcher = chokidar.watch(root)

  const compiler = Compiler.create(options);
  const server = new DevServer({
    root,
    public: 'dist'
  })
  const result= await compiler.build();
  watcher.on('change', (path) => {
    console.log('filechange:',path)
    const module = compiler.moduleGraph.getNodeById(path);
    if(!module){
      return;
    }
    /**
     * @todo update logic
     * 目前会重新触发自该模块开始的全量编译，webpack也是这么做吗
     */
    module.rebuild();
    const content = module.generator();
    const hmrCode = `invalidate(${JSON.stringify(path)})`;
    const sourceUrl = `\n//# sourceURL=${path}`;
    server.broadcast({
      type: 'js-update',
      path: path,
      timestamp: Date.now(),
      code: [content,hmrCode,sourceUrl].join(';') 
    })
  })
  const htmlPath = path.resolve(__dirname, '../index.html');
  for(const [key,value] of Object.entries(result)){
    const dstPath = path.resolve(root,'dist', `${key}.js`);
    fs.ensureFileSync(dstPath);
    fs.writeFileSync(dstPath, value);
  }
  fs.copyFileSync(htmlPath, path.resolve(root, 'dist/index.html'))
  server.start();
}
