// Keep every token and UTF-16 column; only replace newlines with one space. This deliberately
// puts SWC's generated filename and subsequent authored mapped code on the same output line.
const { SourceMapConsumer, SourceMapGenerator } = require('source-map');
module.exports = function(source, map, data) {
  this.experiments.portableOutput();
  const callback = this.async();
  Promise.resolve(new SourceMapConsumer(map)).then(consumer => {
    try {
      const offsets = [0];
      for (const line of source.split('\n')) offsets.push(offsets.at(-1) + line.length + 1);
      const generator = new SourceMapGenerator();
      // SourceMapConsumer exposes URL-encoded names. Preserve the SWC map's exact source
      // identities by their indices; encoding a Unicode filesystem root changes its meaning.
      const sourceNames = new Map(consumer.sources.map((name, index) => [name, map.sources[index]]));
      consumer.eachMapping(mapping => {
        const entry = {
          generated: { line: 1, column: offsets[mapping.generatedLine - 1] + mapping.generatedColumn }
        };
        if (mapping.source != null && mapping.originalLine != null) {
          entry.source = sourceNames.get(mapping.source);
          entry.original = { line: mapping.originalLine, column: mapping.originalColumn };
          if (mapping.name != null) entry.name = mapping.name;
        }
        generator.addMapping(entry);
      });
      for (const [index, name] of map.sources.entries()) {
        generator.setSourceContent(name, map.sourcesContent?.[index] ?? null);
      }
      callback(null, source.replaceAll('\n', ' '), generator.toJSON(), data);
    } catch (error) {
      callback(error);
    } finally {
      consumer.destroy?.();
    }
  }, callback);
};
module.exports.pitch = function() { this.experiments.portableOutput(); };
