import async from 'async';
import { ModuleNode } from './module';
export class AsyncQueue<T>{
  name:string;
  _queue;
  constructor({name,processor}:{name:string, processor: async.AsyncWorker<T,Error>}){
    this.name = name;
    this._queue = async.queue(processor)
    this._queue.drain(function(){
      console.log('finish all module')
    })
    this._queue.error(function(err){
      console.log('err:', err)
    })
  }
  add(item:T,callback:(err?:Error |null |undefined) => void){
    this._queue.push(item, callback)
  }
}