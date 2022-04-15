export class Resolver{
  // resolver: (context:any,id:string,dir:string) => string;
  constructor(){
    // this.resolver = create.sync({
    //   conditions: ['import', 'require', 'node'],
    //   mainFields: ['module','browser', 'main'],
    //   extensions: ['.jsx','.js','.tsx','.ts','json']
    // }) as any;
  }
  resolveRequest(id:string,dir:string){
    return require.resolve(id, {
      paths: [dir]
    })
  }
}