export const checkCacheCounters = (stats, counters) => {
  const logging = stats.toJson({ all: false, logging: "verbose" }).logging;
  const events = (logger, entries) => entries.flatMap(entry => {
    const nested = events(logger, entry.children ?? []);
    if (entry.type !== "cache") return nested;
    const match = /^(.*): [\d.]+% \((\d+)\/(\d+)\)$/.exec(entry.message);
    expect(match).not.toBeNull();
    return [{ logger, label: match[1], hit: Number(match[2]), total: Number(match[3]) }, ...nested];
  });
  const loggedCounters = Object.keys(logging).sort().flatMap(logger => events(logger, logging[logger].entries));
  expect(counters).toEqual(loggedCounters);
};
