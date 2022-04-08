import * as babel from '@babel/core';
export class Parser {
  constructor(){

  }
  parse(content:string){
    const ast = babel.parseSync(content);
    return ast!
  }
}
type fn = typeof babel.parseSync
export type AST = ReturnType<fn>