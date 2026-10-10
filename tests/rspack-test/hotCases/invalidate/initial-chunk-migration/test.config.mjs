import fs from 'node:fs';
import path from 'node:path';

export default {
  moduleScope(scope, _stats, options) {
    scope.readUpdateManifest = hash => {
      const filename = fs.readdirSync(options.output.path).find(file =>
        file.endsWith('.hot-update.json') && file.includes(hash),
      );
      return JSON.parse(fs.readFileSync(path.join(options.output.path, filename), 'utf8'));
    };
  },
};
