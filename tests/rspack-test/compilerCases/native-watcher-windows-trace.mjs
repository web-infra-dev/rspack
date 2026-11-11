import { createRequire } from 'node:module';
import follow from './native-watcher-follow-symlinks.mjs';
import contexts from './native-watcher-symlink-context-recovery.mjs';
import recreated from './native-watcher-symlink-recreated-directory.mjs';

const require = createRequire(import.meta.url);
let tracing = false;

export default [
  follow.find((entry) => entry.name === 'target-roundtrip'),
  contexts.find((entry) => entry.name === 'native-explicit-real'),
  recreated.find((entry) => entry.name === 'native'),
].map((entry) => ({
  name: entry.name,
  description: `trace Windows events for ${entry.name}`,
  skip: process.platform !== 'win32',
  async run() {
    if (!tracing) {
      await require('@rspack/core').experiments.globalTrace.register(
        'rspack_watcher::fs_event=debug',
        'logger',
        'stdout',
      );
      tracing = true;
    }
    console.log('WINDOWS_TRACE_START', entry.name);
    try {
      await entry.run();
    } finally {
      console.log('WINDOWS_TRACE_END', entry.name);
    }
  },
}));
