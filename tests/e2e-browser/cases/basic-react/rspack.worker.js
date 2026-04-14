import { rspack, builtinMemFs } from '@rspack/browser';
import { files, config } from './files.js';

builtinMemFs.volume.fromJSON({
  ...files,
});

try {
  rspack(config, (err, stats) => {
    if (err) {
      self.postMessage({
        type: 'error',
        error: err.message,
      });
      return;
    }

    if (stats?.hasErrors()) {
      self.postMessage({
        type: 'error',
        error: JSON.stringify(stats.toJson().errors),
      });
      return;
    }

    const json = builtinMemFs.volume.toJSON();
    self.postMessage({
      type: 'done',
      output: json['/dist/main.js'],
    });
  });
} catch (error) {
  self.postMessage({
    type: 'error',
    error: error instanceof Error ? error.message : String(error),
  });
}
