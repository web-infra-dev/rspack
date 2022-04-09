import fs from 'fs';
import * as babel from '@babel/core';
import { Resolver } from './resolver';
import { Loader } from './loader';
import { AST, Parser } from './parser';
import { Compiler } from './compiler';
import path from 'path';
export type NormalModuleOptions = {
  path: string;
  resolveDir:string;
  importer: string;
  compiler: Compiler;
  isEntry: boolean;
}
export class ModuleNode {
  isEntry: boolean;
  contents!:string;
  path:string;
  importer:string;
  resolveDir:string; // we need resolveDir to handle virtual Module resolve
  fullPath!: string;
  ast!: AST;
  #resolver!: Resolver;
  #loader!: Loader;
  #parser!: Parser;
  #compiler!: Compiler
  constructor(options:NormalModuleOptions){
    this.path = options.path;
    this.importer = options.importer;
    this.resolveDir = options.resolveDir;
    this.#resolver = new Resolver();
    this.#loader = new Loader();
    this.#compiler = options.compiler;
    this.#parser = new Parser();
    this.isEntry = options.isEntry;
  }
  static create(options:NormalModuleOptions){
    return new ModuleNode(options);
  }
  build(){
    this._doBuild();
  }
  _doBuild(){
    const fullPath = this.#resolver.resolveRequest(this.path, this.resolveDir);
    const contents = this.#loader.load_and_transform(fullPath);
    const ast = this.#parser.parse(contents);
    this.contents = contents;
    this.ast = ast;
    this.fullPath = fullPath;
    this.#compiler.moduleGraph.addNode(fullPath,this);
    this.#compiler.moduleGraph.addEdge(this.importer, fullPath);
    this.buildDeps();
  }
  buildDeps(){
    this.ast?.program.body.forEach(node => {
      if(node.type === 'ImportDeclaration'){
        const moduleId = node.source.value;
        const newModule = ModuleNode.create({
          importer: this.fullPath,
          resolveDir: path.dirname(this.fullPath),
          path: moduleId,
          compiler: this.#compiler,
          isEntry:false
        })
        this.#compiler.addModule(newModule)
      }
    })
  }
}