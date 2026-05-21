import {
  type Config,
  type Plugins,
  type Printer,
  type Refs,
  format as prettyFormat,
  plugins,
} from 'pretty-format';

const {
  AsymmetricMatcher,
  DOMCollection,
  DOMElement,
  Immutable,
  ReactElement,
  ReactTestComponent,
} = plugins;

const mockSerializer = {
  test(val: unknown) {
    return Boolean(
      val &&
      typeof val === 'object' &&
      (val as { _isMockFunction?: boolean })._isMockFunction,
    );
  },
  serialize(
    val: {
      getMockName(): string;
      mock: { calls: unknown[]; results: unknown[] };
    },
    config: Config,
    indentation: string,
    depth: number,
    refs: Refs,
    printer: Printer,
  ) {
    const name = val.getMockName();
    const nameString = name === 'jest.fn()' ? '' : ` ${name}`;
    let callsString = '';

    if (val.mock.calls.length !== 0) {
      const indentationNext = indentation + config.indent;
      callsString = ` {${config.spacingOuter}${indentationNext}"calls": ${printer(
        val.mock.calls,
        config,
        indentationNext,
        depth,
        refs,
      )}${config.min ? ', ' : ','}${
        config.spacingOuter
      }${indentationNext}"results": ${printer(
        val.mock.results,
        config,
        indentationNext,
        depth,
        refs,
      )}${config.min ? '' : ','}${config.spacingOuter}${indentation}}`;
    }

    return `[MockFunction${nameString}]${callsString}`;
  },
};

export const getSnapshotSerializers = (): Plugins => [
  ReactTestComponent,
  ReactElement,
  DOMElement,
  DOMCollection,
  Immutable,
  mockSerializer,
  AsymmetricMatcher,
];

export const normalizeNewlines = (str: string) => str.replace(/\r\n|\r/g, '\n');

export const serializeSnapshot = (
  val: unknown,
  indent = 2,
  formatOverrides = {},
) =>
  normalizeNewlines(
    prettyFormat(val, {
      escapeRegex: true,
      indent,
      plugins: getSnapshotSerializers(),
      printFunctionName: false,
      ...formatOverrides,
    }),
  );
