
import { Loader } from './loader';
import { Resolver } from './resolver';
import { AsyncQueue } from './queue';
import { ModuleNode, NormalModuleOptions } from './module';
import Module from 'module';
import { ModuleGraph } from './module-graph';
import { Bundler } from './bundle';
function noop(){

}
type BuildCallback = (module:ModuleNode, err?:Error) => void;

export class Compiler{
  entry:string;
  root: string;
  loader: Loader;
  resolver: Resolver;
  buildQueue: AsyncQueue<ModuleNode>;
  moduleGraph: ModuleGraph;
  private constructor({entry, root}:{entry:string,root:string}){
    this.entry = entry;
    this.loader = new Loader();
    this.resolver = new Resolver();
    this.root = root;
    this.buildQueue = new AsyncQueue({
      name: 'build',
      processor: this._buildModule.bind(this)
    })
    this.moduleGraph = new ModuleGraph();
  }
  static create(options:{
    entry:string,
    root:string
  }){
    return new Compiler(options);
  }
  _buildModule(mod: ModuleNode,done:Function){
    console.log('mod',mod)
    mod.build();
    done()
  }
  build(){
    this.addModule(ModuleNode.create({
      path: this.entry,
      resolveDir: this.root,
      importer: '',
      compiler: this,
      isEntry: true
    }));
  }
  buildModule(module: ModuleNode){
    this.buildQueue.add(module,(err?) => {
      if(err){
        console.error('build module failed', err)
      }else {
        //console.info('build module success:', module.path)
      }
    })
  }
  addModule(module: ModuleNode){
    this.buildModule(module,);
  }
  generate(){
    const bundler = new Bundler();
    bundler.bundle(this.moduleGraph);
  }
}

export async function build(entry:string){
  const compiler = Compiler.create({
    entry: entry,
    root: ''
  });
  await compiler.build();
  await compiler.generate();
}

