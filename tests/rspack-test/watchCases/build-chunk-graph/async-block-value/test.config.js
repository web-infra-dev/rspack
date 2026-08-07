function assert(condition, message) {
  if (!condition) {
    throw new Error(`Assertion failed for ${message}`);
  }
}

function shouldRebuild(stats) {
  return stats.includes('<t> rebuild chunk graph');
}

module.exports = {
  checkStats(stepName, _, stats) {
    switch (stepName) {
      case '0':
        assert(
          shouldRebuild(stats),
          'initial compilation should build chunk graph',
        );
        break;
      case '1':
        assert(
          !shouldRebuild(stats),
          'loc-only change should reuse chunk graph',
        );
        break;
      case '2':
        assert(
          shouldRebuild(stats),
          'async block target change should rebuild chunk graph',
        );
        assert(
          stats.includes('module code splitting value changed'),
          'async block value comparison should detect the target change',
        );
        break;
      default:
        throw new Error(`Unexpected step ${stepName}`);
    }

    return true;
  },
};
