import fs from 'node:fs';
import path from 'node:path';

export default {
  moduleScope(scope, _stats, options) {
    scope.BUILD_REPORT = JSON.parse(
      fs.readFileSync(
        path.join(options.output.path, `${options.name}.modules.json`),
        'utf8',
      ),
    );
  },
};
