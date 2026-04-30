const fs = require('node:fs');
const path = require('node:path');

let outputPath;

module.exports = {
  findBundle(index, options) {
    if (index === 0) {
      outputPath = options.output.path;
    }
    return [];
  },
  checkStats(stepName) {
    const logFile = path.join(outputPath, 'on-server-component-changes.log');
    const log = fs.existsSync(logFile) ? fs.readFileSync(logFile, 'utf-8') : '';

    switch (stepName) {
      case '0':
        expect(log).toBe('');
        break;
      case '1':
        expect(log).toBe('callback returned void\n');
        break;
      case '2':
        expect(log).toBe(
          'callback returned void\ncallback resolved promise\n',
        );
        break;
      default:
        throw new Error(`Unexpected step: ${stepName}`);
    }

    return true;
  },
};
