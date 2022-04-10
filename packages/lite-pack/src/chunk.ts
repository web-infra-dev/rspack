
export class Chunk {
  id:string;
  modules: string[];
  constructor(options: {
    id:string,
  }){
    this.modules = [];
    this.id = options.id;
  }
  render():string{
    return ''
  }
}