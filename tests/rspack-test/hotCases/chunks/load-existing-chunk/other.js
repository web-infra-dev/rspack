import { value } from './shared';
globalThis.loadOtherValue = value;
module.hot.accept('./shared', () => {
  globalThis.loadOtherValue = value;
});
