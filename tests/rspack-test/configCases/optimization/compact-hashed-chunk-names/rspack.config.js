module.exports = [false, true].map((uppercase) => {
  // Reserve every possible one-character id without depending on a specific hash.
  const names = Array.from(
    uppercase
      ? '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ'
      : '0123456789abcdefghijklmnopqrstuvwxyz',
  );
  return {
    entry: Object.fromEntries(names.map((name) => [name, './index.js'])),
    output: {
      filename: `${uppercase ? 'upper' : 'lower'}/[name].js`,
      chunkFilename: `${uppercase ? 'upper' : 'lower'}/[name].js`,
    },
    optimization: {
      chunkIds: 'compact-hashed',
    },
    plugins: [
      (compiler) => {
        compiler.hooks.done.tap('CheckReservedChunkNames', (stats) => {
          const chunks = stats.toJson({
            all: false,
            chunks: true,
            ids: true,
          }).chunks;
          const reserved = new Set(names.map((name) => name.toLowerCase()));
          expect(chunks.some((chunk) => !chunk.names.length)).toBe(true);
          for (const chunk of chunks) {
            expect(reserved.has(String(chunk.id).toLowerCase())).toBe(false);
            expect(String(chunk.id).length).toBeGreaterThan(1);
          }
          expect(
            new Set(
              chunks.flatMap((chunk) =>
                chunk.files.map((file) => file.toLowerCase()),
              ),
            ).size,
          ).toBe(chunks.length);
        });
      },
    ],
  };
});
