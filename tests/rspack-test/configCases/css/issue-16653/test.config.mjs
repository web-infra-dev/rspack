import { readFileSync } from 'node:fs';
import { join } from 'node:path';

export default {
  afterExecute(options) {
    for (const config of options) {
      let source = readFileSync(
        join(config.output.path, config.output.filename),
        'utf8',
      );
      const exportType = config.module.rules[0].parser.exportType;
      if (exportType === undefined || exportType === 'link') {
        source += readFileSync(
          join(config.output.path, config.output.cssFilename),
          'utf8',
        );
      }
      // Each CSS-only URL belongs in the rendered CSS exactly once, with no
      // duplicate in a JavaScript asset module factory.
      const icon =
        config.module.rules[1].type === 'asset/inline'
          ? readFileSync(new URL('./icon.svg', import.meta.url)).toString('base64')
          : 'icon.svg';
      expect(source.split(icon)).toHaveLength(2);
      expect(source.split('data:image/svg+xml,%3Csvg%3E%3C/svg%3E')).toHaveLength(2);
    }
  },
};
