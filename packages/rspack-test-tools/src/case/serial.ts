import { BasicCaseCreator } from '../test/creator';
import { createConfigProcessor } from './config';
import { createMultiCompilerRunner, getMultiCompilerRunnerKey } from './runner';

const creator = new BasicCaseCreator({
  clean: true,
  describe: false,
  testConfig: (testConfig) => {
    const oldModuleScope = testConfig.moduleScope;
    testConfig.moduleScope = (ms, stats, compilerOptions) => {
      let res = ms;
      // TODO: modify runner module scope based on stats here
      if (typeof oldModuleScope === 'function') {
        res = oldModuleScope(ms, stats, compilerOptions);
      }
      return res;
    };
  },
  steps: ({ name }) => [createConfigProcessor(name)],
  runner: {
    key: getMultiCompilerRunnerKey,
    runner: createMultiCompilerRunner,
  },
  concurrent: 1,
});

export function createSerialCase(name: string, src: string, dist: string) {
  creator.create(name, src, dist);
}
