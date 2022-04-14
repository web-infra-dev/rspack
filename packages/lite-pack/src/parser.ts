import * as babel from '@babel/core';
export class Parser {
  constructor(){

  }
  parse(content:string){
    const ast = babel.parseSync(content, {
      plugins: [{
        visitor: {
          Identifier(path){
            console.log('path:',path);
          },
          ImportDeclaration(node){
            console.log('node:',node);
          }
        }
      }]
    });
    return ast!
  }
}
type fn = typeof babel.parseSync
export type AST = ReturnType<fn>