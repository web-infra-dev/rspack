import EventEmitter from 'node:events';
import { Compiler, type RspackOptions, type Stats } from '@rspack/core';
import merge from 'webpack-merge';
import { DEBUG_SCOPES } from './test/debug';
import type { ITestCompilerManager, ITestContext } from './type';

export enum ECompilerEvent {
  Build = 'build',
  Option = 'option',
  Create = 'create',
  Close = 'close',
}

export class TestCompilerManager implements ITestCompilerManager {
  protected compilerOptions: RspackOptions = {} as RspackOptions;
  protected compilerInstance: Compiler | null = null;
  protected compilerStats: Stats | null = null;
  protected emitter: EventEmitter = new EventEmitter();

  constructor(protected context: ITestContext) {}

  getOptions(): RspackOptions {
    return this.compilerOptions;
  }

  setOptions(newOptions: RspackOptions): RspackOptions {
    this.compilerOptions = newOptions;
    this.emitter.emit(ECompilerEvent.Option, this.compilerOptions);
    return this.compilerOptions;
  }

  mergeOptions(newOptions: RspackOptions): RspackOptions {
    this.compilerOptions = merge(this.compilerOptions, newOptions);
    this.emitter.emit(ECompilerEvent.Option, this.compilerOptions);
    return this.compilerOptions;
  }

  getCompiler(): Compiler | null {
    return this.compilerInstance;
  }

  createCompiler(): Compiler {
    this.compilerInstance = require('@rspack/core')(
      this.compilerOptions,
    ) as Compiler;
    if (__DEBUG__) {
      const context = this.context;
      this.compilerInstance = new Proxy(this.compilerInstance, {
        get(target, p, receiver) {
          const value = Reflect.get(target, p, receiver);
          if (
            typeof value === 'function' &&
            Compiler.prototype.hasOwnProperty(p)
          ) {
            return value.bind(target);
          }
          return value;
        },
        set(target, p, value, receiver) {
          const debugSetProperties =
            (context.getValue(
              DEBUG_SCOPES.CreateCompilerSetProperties,
            ) as string[]) || [];
          debugSetProperties.push(
            `${p as string} ${new Error().stack?.split('\n')[2]?.trim()}`,
          );
          context.setValue(
            DEBUG_SCOPES.CreateCompilerSetProperties,
            debugSetProperties,
          );
          return Reflect.set(target, p, value, receiver);
        },
      });
      this.context.setValue(DEBUG_SCOPES.CreateCompilerInstance, {
        path: require.resolve('@rspack/core'),
        mode: 'no-callback',
      });
    }
    this.emitter.emit(ECompilerEvent.Create, this.compilerInstance);
    return this.compilerInstance;
  }

  createCompilerWithCallback(
    callback: (error: Error | null, stats: Stats | null) => void,
  ): Compiler {
    this.compilerInstance = require('@rspack/core')(
      this.compilerOptions,
      callback,
    ) as Compiler;
    if (__DEBUG__) {
      const context = this.context;
      this.compilerInstance = new Proxy(this.compilerInstance, {
        get(target, p, receiver) {
          const value = Reflect.get(target, p, receiver);
          if (
            typeof value === 'function' &&
            Compiler.prototype.hasOwnProperty(p)
          ) {
            return value.bind(target);
          }
          return value;
        },
        set(target, p, value, receiver) {
          const debugSetProperties =
            (context.getValue(
              DEBUG_SCOPES.CreateCompilerSetProperties,
            ) as string[]) || [];
          debugSetProperties.push(
            `${p as string} ${new Error().stack?.split('\n')[2]?.trim()}`,
          );
          context.setValue(
            DEBUG_SCOPES.CreateCompilerSetProperties,
            debugSetProperties,
          );
          return Reflect.set(target, p, value, receiver);
        },
      });
      this.context.setValue(DEBUG_SCOPES.CreateCompilerInstance, {
        path: require.resolve('@rspack/core'),
        mode: 'callback',
      });
    }
    this.emitter.emit(ECompilerEvent.Create, this.compilerInstance);
    return this.compilerInstance;
  }

  build(): Promise<Stats> {
    if (!this.compilerInstance)
      throw new Error('Compiler should be created before build');
    return new Promise<Stats>((resolve, reject) => {
      try {
        const context = this.context;
        if (__DEBUG__) {
          context.setValue(DEBUG_SCOPES.BuildMethod, {
            method: 'run',
          });
        }
        this.compilerInstance!.run((error, newStats) => {
          this.emitter.emit(ECompilerEvent.Build, error, newStats);
          if (error) {
            if (__DEBUG__) {
              context.setValue(DEBUG_SCOPES.BuildError, {
                type: 'fatal',
                errors: [error],
              });
            }
            return reject(error);
          }
          this.compilerStats = newStats as Stats;
          if (__DEBUG__) {
            if (newStats?.hasErrors()) {
              context.setValue(DEBUG_SCOPES.BuildError, {
                type: 'stats',
                errors:
                  newStats.toJson({
                    errors: true,
                  }).errors || [],
              });
            }
            if (newStats?.hasWarnings()) {
              context.setValue(
                DEBUG_SCOPES.BuildWarning,
                newStats.toJson({
                  warnings: true,
                }).warnings || [],
              );
            }
          }

          resolve(newStats as Stats);
        });
      } catch (e) {
        reject(e);
      }
    });
  }

  watch(timeout = 1000) {
    if (!this.compilerInstance)
      throw new Error('Compiler should be created before watch');
    const context = this.context;
    const watchOptions = {
      // IMPORTANT:
      // This is a workaround for the issue that watchpack cannot detect the file change in time
      // so we set the poll to 300ms to make it more sensitive to the file change
      poll: 300,
      // Rspack ignored node_modules and .git by default for better performance, but for tests we
      // want to watch all files, which aligns with webpack's default behavior
      ignored: [],
      aggregateTimeout: timeout,
    };
    if (__DEBUG__) {
      context.setValue(DEBUG_SCOPES.BuildMethod, {
        method: 'watch',
        options: watchOptions,
      });
    }
    this.compilerInstance!.watch(watchOptions, (error, newStats) => {
      this.emitter.emit(ECompilerEvent.Build, error, newStats);
      if (__DEBUG__) {
        if (error) {
          context.setValue(DEBUG_SCOPES.BuildError, {
            type: 'fatal',
            errors: [error],
          });
          return error;
        }
      }

      if (newStats) {
        if (__DEBUG__) {
          if (newStats.hasErrors()) {
            context.setValue(DEBUG_SCOPES.BuildError, {
              type: 'stats',
              errors:
                newStats.toJson({
                  errors: true,
                }).errors || [],
            });
          }
          if (newStats.hasWarnings()) {
            context.setValue(
              DEBUG_SCOPES.BuildWarning,
              newStats.toJson({
                warnings: true,
              }).warnings || [],
            );
          }
        }

        this.compilerStats = newStats as Stats;
      }
      return newStats;
    });
  }

  getStats() {
    return this.compilerStats;
  }

  getEmitter() {
    return this.emitter;
  }

  close(): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      if (this.compilerInstance) {
        this.compilerInstance.close((e) => {
          this.emitter.emit(ECompilerEvent.Close, e);
          e ? reject(e) : resolve();
        });
      } else {
        resolve();
      }
    });
  }
}
