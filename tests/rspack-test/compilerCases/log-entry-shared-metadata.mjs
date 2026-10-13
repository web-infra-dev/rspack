let compilations = 0;
let previousTrace;
let previousNoTrace;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description:
    'shares log accessors without changing lazy trace or hook semantics',
  options() {
    return {
      entry: './d',
      cache: true,
      plugins: [
        (compiler) => {
          compiler.hooks.compilation.tap(
            'SharedLogMetadata',
            (compilation) => {
              compilations++;
              const observed = [];
              compilation.hooks.log.tap(
                'SharedLogMetadata',
                (name, entry) => {
                  if (!name.startsWith('shared-log')) return;
                  observed.push(entry);
                  expect(Reflect.ownKeys(entry)).toEqual([
                    'time',
                    'type',
                    'args',
                    'trace',
                  ]);
                  expect(Object.getPrototypeOf(entry)).toBe(Object.prototype);
                  for (const key of ['time', 'type', 'args']) {
                    expect(
                      Object.getOwnPropertyDescriptor(entry, key),
                    ).toMatchObject({
                      writable: true,
                      enumerable: true,
                      configurable: true,
                    });
                  }
                  const descriptor = Object.getOwnPropertyDescriptor(
                    entry,
                    'trace',
                  );
                  expect(descriptor.enumerable).toBe(true);
                  expect(descriptor.configurable).toBe(true);
                  expect(descriptor.set).toBeUndefined();
                  expect(descriptor.get.name).toBe('get trace');
                  expect(descriptor.get.length).toBe(0);
                  expect(typeof entry.time).toBe('number');
                  if (name === 'shared-log/mutated') {
                    entry.type = entry.type === 'warn' ? 'info' : 'warn';
                  }
                  if (name === 'shared-log/blocked') return true;
                },
              );

              const logger = compilation.getLogger('shared-log');
              const payload = { retained: true };
              logger.warn(payload);
              logger.error('error');
              logger.trace();
              logger.info('info');
              logger.log('log');
              logger.debug('debug');
              logger.group('group');
              logger.groupCollapsed('collapsed');
              logger.groupEnd();
              logger.status('status');
              logger.clear();
              const entries = compilation.logging.get('shared-log');
              expect(entries).toEqual(observed);
              expect(entries.map((entry) => entry.type)).toEqual([
                'warn',
                'error',
                'trace',
                'info',
                'log',
                'debug',
                'group',
                'groupCollapsed',
                'groupEnd',
                'status',
                'clear',
              ]);
              expect(entries[0].args[0]).toBe(payload);
              expect(entries[10].args).toBeUndefined();

              const getTrace = Object.getOwnPropertyDescriptor(
                entries[0],
                'trace',
              ).get;
              const getNoTrace = Object.getOwnPropertyDescriptor(
                entries[3],
                'trace',
              ).get;
              expect(getTrace).not.toBe(getNoTrace);
              for (const entry of entries.slice(0, 3)) {
                expect(
                  Object.getOwnPropertyDescriptor(entry, 'trace').get,
                ).toBe(getTrace);
              }
              for (const entry of entries.slice(3)) {
                expect(
                  Object.getOwnPropertyDescriptor(entry, 'trace').get,
                ).toBe(getNoTrace);
                expect(entry.trace).toBeUndefined();
              }
              if (previousTrace) {
                expect(getTrace).toBe(previousTrace);
                expect(getNoTrace).toBe(previousNoTrace);
              }
              previousTrace = getTrace;
              previousNoTrace = getNoTrace;

              // Trace eligibility belongs to the original call, not a hook-mutated
              // type or the receiver used when an accessor is extracted.
              const mutated = logger.getChildLogger('mutated');
              mutated.warn('was warn');
              mutated.info('was info');
              const changed = compilation.logging.get('shared-log/mutated');
              expect(changed.map((entry) => entry.type)).toEqual([
                'info',
                'warn',
              ]);
              expect(
                Object.getOwnPropertyDescriptor(changed[0], 'trace').get,
              ).toBe(getTrace);
              expect(
                Object.getOwnPropertyDescriptor(changed[1], 'trace').get,
              ).toBe(getNoTrace);
              expect(getNoTrace.call(changed[0])).toBeUndefined();

              // Preserve read-time stack generation, repeated reads, stack trimming,
              // and getters on sealed/frozen or otherwise modified records.
              const prepareStackTrace = Error.prepareStackTrace;
              let stackReads = 0;
              try {
                Error.prepareStackTrace = () => {
                  stackReads++;
                  return 'Error: Trace\n getter\n reader\n kept\n LOADER_EXECUTION\n omitted';
                };
                logger.warn('lazy');
                expect(stackReads).toBe(0);
                const lazy = entries.at(-1);
                Object.freeze(lazy);
                Object.seal(changed[0]);
                expect(lazy.trace).toEqual([' kept']);
                expect(lazy.trace).toEqual([' kept']);
                expect(changed[0].trace).toEqual([' kept']);
                expect(changed[1].trace).toBeUndefined();
                expect(getTrace.call(null)).toEqual([' kept']);
                expect(stackReads).toBe(4);
              } finally {
                if (prepareStackTrace === undefined)
                  delete Error.prepareStackTrace;
                else Error.prepareStackTrace = prepareStackTrace;
              }

              logger.getChildLogger('blocked').warn('not stored');
              expect(compilation.logging.has('shared-log/blocked')).toBe(
                false,
              );
              expect(
                Object.getOwnPropertyDescriptor(observed.at(-1), 'trace').get,
              ).toBe(getTrace);
              // Accessors remain configurable own properties, not prototype fields.
              Object.defineProperty(entries[1], 'trace', {
                value: ['replacement'],
              });
              expect(entries[1].trace).toEqual(['replacement']);
              expect(delete entries[2].trace).toBe(true);
              expect(Object.hasOwn(entries[2], 'trace')).toBe(false);
              expect(() => {
                entries[3].trace = [];
              }).toThrow(TypeError);
            },
          );
        },
      ],
    };
  },
  async build(_, compiler) {
    for (let i = 0; i < 2; i++) {
      await new Promise((resolve, reject) => {
        compiler.run((error, stats) => {
          if (error) return reject(error);
          try {
            expect(stats.hasErrors()).toBe(false);
            const logging = stats.toJson({
              all: false,
              logging: 'verbose',
            }).logging;
            expect(logging['shared-log'].entries[0].message).toContain(
              'retained',
            );
            resolve();
          } catch (error) {
            reject(error);
          }
        });
      });
    }
  },
  async check() {
    expect(compilations).toBe(2);
  },
};
