import fs from 'fs';
/**
 * @todo support plugins
 */
export class Loader {
  load(fileName:string):string{
    const contents = fs.readFileSync(fileName,'utf-8');
    return contents;
  }
  transform(content:string){
    return content;
  }
  load_and_transform(fileName:string){
    const result = this.load(fileName);
    const transformResult = this.transform(result);
    return transformResult;
  }
}