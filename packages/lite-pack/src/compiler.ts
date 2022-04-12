import { Loader } from './loader';
import { Resolver } from './resolver';
import { AsyncQueue } from './queue';
import { ModuleNode, NormalModuleOptions } from './module';
import fs from 'fs-extra';
import path from 'path';
import { ModuleGraph } from './module-graph';
import { Bundler } from './bundle';
import { DevServer } from './server';
import { } from 'ws';
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
  entry: Record<string, string>;
  root: string;
  loader: Loader;
  resolver: Resolver;
  buildQueue: AsyncQueue<ModuleNode>;
  moduleGraph: ModuleGraph;
  private constructor({ entry, root }: { entry: Record<string, string>; root: string }) {
    this.entry = entry;
    this.loader = new Loader();
    this.resolver = new Resolver();
    this.root = root;
    this.buildQueue = new AsyncQueue({
      name: 'build',
      processor: this._buildModule.bind(this),
    });
    this.moduleGraph = new ModuleGraph();
  }
  static create(options: { entry: Record<string, string>; root: string }) {
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
          entryKey: key
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
    const bundler = new Bundler(this.moduleGraph);
    bundler.build();
    return bundler.output;
  }
  async build(){
    await this.generate_module_graph();
    return this.generate();
  }
}

export async function build(entry: Record<string,string>) {
  const root = path.resolve(__dirname,'..');
  const server = new DevServer({
    root,
    public: 'dist'
  })
  const compiler = Compiler.create({
    entry: entry,
    root,
  });
  const result= await compiler.build();
  for(const [key,value] of Object.entries(result)){
    const dstPath = path.resolve(root,'dist', `${key}.js`);
    fs.ensureFileSync(dstPath);
    fs.writeFileSync(dstPath, value);
  }
  server.start();
}
