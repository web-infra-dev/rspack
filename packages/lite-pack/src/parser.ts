import * as babel from '@babel/core';
import traverse from '@babel/traverse'
import { ImportKind } from 'esbuild';
export type ImportType = {
  kind: ImportKind
  id: string;
}
export class Parser {
  constructor(){

  }
  parse(content:string){
    const ast = babel.parseSync(content)!;
    const imports: ImportType[] = []
    traverse(ast, {
      CallExpression:({node}) => {
        if(node.callee.type === 'Import'){
          const argument = node.arguments[0];
          if(argument.type === 'StringLiteral'){
            imports.push({
              kind: 'dynamic-import',
              id: argument.value
            })
          }
        }
      },
      ImportDeclaration:({node}) => {
        const id = node.source.value;
        imports.push({
          kind: 'import-statement',
          id: id
        })
      }
    })
    return {ast,imports}
  }
}
type fn = typeof babel.parseSync
export type AST = ReturnType<fn>