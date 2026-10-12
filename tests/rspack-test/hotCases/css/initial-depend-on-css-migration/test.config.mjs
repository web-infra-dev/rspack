import fs from 'node:fs';

export default {
  findBundle(_index, options) {
    const files = fs.readdirSync(options.output.path);
    return [
      files.find(file => /^base\..*\.css$/.test(file)),
      files.find(file => /^main\..*\.css$/.test(file)),
      'runtime.js', 'base.js', 'parent.js', 'sibling.js', 'main.js',
    ];
  },
};
