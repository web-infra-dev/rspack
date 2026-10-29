import path from 'node:path';
import { SourceMapGenerator } from 'source-map';

export default function (code) {
  const map = new SourceMapGenerator({ file: this.resourcePath });
  const source = path.join(this.context, 'original.less').replace(/\\/g, '/');
  for (let line = 1; line <= code.split('\n').length; line++) {
    map.addMapping({ generated: { line, column: 0 }, original: { line: line + 2, column: 0 }, source });
  }
  map.setSourceContent(source, `// original source\n// second line\n${code}`);
  this.callback(null, code, map.toJSON(), { preserved: true });
}
