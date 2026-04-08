import { importedOnly } from './dep';

function localOnly() {
  globalThis.sideEffectCount += 1;
}

localOnly();
importedOnly();
notExistFunction();
