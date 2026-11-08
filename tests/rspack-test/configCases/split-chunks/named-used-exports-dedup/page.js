import { track } from './modules/singleton';
import { helper } from './modules/helper';

globalThis.result = [track(), helper()];
