import fs from 'fs';
import * as babel from '@babel/core';
import { Resolver } from './resolver';
import { Loader } from './loader';
import { AST, ImportType, Parser } from './parser';
import { Compiler } from './compiler';
import path from 'path';
import { Chunk } from './chunk';
import * as esbuild from 'esbuild';
export type NormalModuleOptions = {
  path: string;
  resolveDir: string;
  importer: string;
  compiler: Compiler;
  isEntry: boolean;
  entryKey?: string;
};
export class ModuleNode {
  isEntry: boolean;
  entryKey?: string;
  contents!: string;
  path: string;
  importer: string;
  resolveDir: string; // we need resolveDir to handle virtual Module resolve
  fullPath!: string;
  ast!: AST;
  #resolver!: Resolver;
  #loader!: Loader;
  #parser!: Parser;
  #compiler!: Compiler;
  chunks: Set<Chunk> = new Set();
  depMap: Map<string, string> = new Map();
  imports: ImportType[] = [];
  constructor(options: NormalModuleOptions) {
    this.path = options.path;
    this.importer = options.importer;
    this.resolveDir = options.resolveDir;
    this.#resolver = new Resolver();
    this.#loader = new Loader();
    this.#compiler = options.compiler;
    this.#parser = new Parser();
    this.isEntry = options.isEntry;
    this.entryKey = options.entryKey;
  }
  static create(options: NormalModuleOptions) {
    return new ModuleNode(options);
  }
  build() {
    this._doBuild();
  }
  rebuild() {
    this._doBuild();
  }
  _doBuild() {
    const fullPath = this.#resolver.resolveRequest(this.path, this.resolveDir);
    const contents = this.#loader.load_and_transform(fullPath);
    const { imports, ast } = this.#parser.parse(contents);
    this.imports = imports;
    this.contents = contents;
    this.ast = ast;
    this.fullPath = fullPath;
    this.#compiler.moduleGraph.addNode(fullPath, this);
    const importerModule = this.#compiler.moduleGraph.getNodeById(this.importer);
    importerModule?.depMap.set(this.path, fullPath);
    this.#compiler.moduleGraph.addEdge(this.importer, fullPath);
    this.buildDeps();
  }
  buildDeps() {
    console.log('imports:', this.imports);
    for (const record of this.imports) {
      const moduleId = record.id;
      const newModule = ModuleNode.create({
        importer: this.fullPath,
        resolveDir: path.dirname(this.fullPath),
        path: moduleId,
        compiler: this.#compiler,
        isEntry: false,
      });
      this.#compiler.addModule(newModule);
    }
  }
  generator() {
    const { types: t } = babel;
    const self = this;
    const code = babel.transformSync(this.contents, {
      plugins: [
        function () {
          return {
            visitor: {
              CallExpression:({node}) => {
                if(node.callee.type === 'Import'){
                  const argument = node.arguments[0];
                  if(argument.type === 'StringLiteral'){
                    const id = argument.value;
                    const replaceId = self.depMap.get(id);
                    console.log('xxxx:', replaceId);
                    node.arguments[0] = t.stringLiteral(replaceId!);
                  }
                }
              },
              ImportDeclaration(path) {
                const newIdentifier = path.scope.generateUidIdentifier('imported');

                for (const specifier of path.get('specifiers')) {
                  const binding = specifier.scope.getBinding(specifier.node.local.name);
                  const importedKey = specifier.isImportDefaultSpecifier()
                    ? 'default'
                    : specifier.get('imported.name').node;

                  for (const referencePath of binding.referencePaths) {
                    referencePath.replaceWith(t.memberExpression(newIdentifier, t.stringLiteral(importedKey), true));
                  }
                }
                const importPath: string = path.get('source.value').node;
                const importerPath = self.depMap.get(importPath);
                path.replaceWith(
                  t.variableDeclaration('const', [
                    t.variableDeclarator(
                      newIdentifier,
                      t.callExpression(t.identifier('require'), [t.stringLiteral(importerPath)])
                    ),
                  ])
                );
              },
            },
          } as babel.PluginObj;
        },
        require('@babel/plugin-transform-modules-commonjs'),
        require('@babel/plugin-proposal-dynamic-import'),
      ],
    })!.code!;
    console.log('xxx:', code);
    return `rs.define(${JSON.stringify(this.fullPath)},function test(require,exports,module){${code}});`;
  }
  addChunk(chunk: Chunk) {
    this.chunks.add(chunk);
  }
}
