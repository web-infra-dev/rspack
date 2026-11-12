import { helpers } from '@module-federation/runtime-tools/runtime';

export default () => ({
  name: 'verify-runtime-helper-subpaths',
  beforeInit(args) {
    if (typeof helpers.utils.getRemoteInfo !== 'function') {
      throw new Error('Runtime helpers are unavailable during initialization');
    }
    globalThis.__mfRuntimeHelpersReady = true;
    return args;
  },
});
