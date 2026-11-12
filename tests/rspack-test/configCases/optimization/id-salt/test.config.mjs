export default {
  findBundle: () => [],
  validate({ stats }) {
    const ids = stats.map(child => {
      const { chunks, modules } = child.toJson({
        all: false,
        chunks: true,
        modules: true,
        ids: true
      });
      return {
        chunks: Object.fromEntries(chunks.map(c => [c.names[0], c.id])),
        modules: Object.fromEntries(
          modules.filter(m => m.name?.startsWith("./")).map(m => [m.name, m.id])
        )
      };
    });
    for (const offset of [0, 4]) {
      const [builtin, explicitDefault, salted, repeated] = ids.slice(offset);
      expect(explicitDefault).toEqual(builtin);
      expect(repeated).toEqual(salted);
      expect(salted.chunks).not.toEqual(builtin.chunks);
      expect(salted.modules).not.toEqual(builtin.modules);
    }
  }
};
