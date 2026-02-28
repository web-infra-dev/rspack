import fs from 'fs-extra';
import type {
  ITestContext,
  ITestEnv,
  ITester,
  ITesterConfig,
  ITestProcessor,
} from '../type';
import { TestContext } from './context';
import { generateDebugReport } from './debug';

export class Tester implements ITester {
  private context: ITestContext;
  private steps: ITestProcessor[] = [];
  step = 0;
  total = 0;

  constructor(private config: ITesterConfig) {
    this.context = config.createContext
      ? config.createContext(config)
      : new TestContext(config);
    this.steps = config.steps || [];
    this.step = 0;
    this.total = config.steps?.length || 0;
    if (config.contextValue) {
      for (const [key, value] of Array.from(
        Object.entries(config.contextValue),
      )) {
        this.context.setValue(key, value);
      }
    }
  }
  getContext(): ITestContext {
    return this.context;
  }
  async prepare() {
    fs.mkdirSync(this.context.getDist(), { recursive: true });
    const tempDir = this.context.getTemp();
    if (tempDir) {
      fs.mkdirSync(tempDir, { recursive: true });
    }
    for (const i of this.steps) {
      if (typeof i.beforeAll === 'function') {
        await i.beforeAll(this.context);
      }
    }
  }
  async compile() {
    const currentStep = this.steps[this.step];
    if (!currentStep) return;

    await this.runStepMethods(currentStep, [
      'before',
      'config',
      'compiler',
      'build',
    ]);
  }
  async check(env: ITestEnv) {
    const currentStep = this.steps[this.step];
    if (!currentStep) return;

    await this.runCheckStepMethods(
      currentStep,
      env,
      this.context.hasError() ? ['check'] : ['run', 'check'],
    );
  }

  async after() {
    const currentStep = this.steps[this.step];
    if (!currentStep) return;
    await this.runStepMethods(currentStep, ['after'], true);
  }

  next() {
    if (this.context.hasError()) {
      return false;
    }
    if (this.steps[this.step + 1]) {
      this.step++;
      return true;
    }
    return false;
  }

  async resume() {
    for (const i of this.steps) {
      if (typeof i.afterAll === 'function') {
        await i.afterAll(this.context);
      }
    }
    try {
      await this.context.closeCompiler();
    } catch (e: any) {
      console.warn(
        `Error occured while closing compilers of '${this.config.name}':\n${e.stack}`,
      );
    }
    if (__DEBUG__) {
      try {
        generateDebugReport(this.context);
      } catch (e) {
        console.warn(`Generate debug report failed: ${(e as Error).message}`);
      }
    }
  }

  private async runStepMethods(
    step: ITestProcessor,
    methods: Array<'before' | 'config' | 'compiler' | 'build' | 'after'>,
    force = false,
  ) {
    for (const i of methods) {
      if (!force && this.context.hasError()) return;
      if (typeof step[i] === 'function') {
        try {
          await step[i]!(this.context);
        } catch (e) {
          this.context.emitError(e as Error);
        }
      }
    }
  }

  private async runCheckStepMethods(
    step: ITestProcessor,
    env: ITestEnv,
    methods: Array<'run' | 'check'>,
  ) {
    try {
      for (const i of methods) {
        if (typeof step[i] === 'function') {
          await step[i]!(env, this.context);
        }
      }
    } catch (e) {
      const errors = this.context.getError();
      console.error(
        new Error([...errors, e].map((e) => (e as Error).message).join('\n')),
      );
      throw e;
    }
  }
}
