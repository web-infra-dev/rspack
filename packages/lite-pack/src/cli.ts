import path from 'path';
import { build } from './compiler';


 build({main:path.resolve(__dirname,'../fixtures/index.js')})