import { defineConfig } from '@rspack/cli';

export default ['sync', 'async', 'mixed'].map((mode) => {
  const condition = (regexp: RegExp) =>
    mode === 'async' ? (value: string) => regexp.test(value) : regexp;
  const use = (prefix: string) => ({
    loader: './prefix-loader.mjs',
    options: { prefix },
  });

  return defineConfig({
    module: {
      rules: [
        { test: condition(/\.txt$/), use: use('wrong:') },
        { test: condition(/never-match/), as: 'wrong.txt' },
        {
          test: condition(/(?:example|virtual)\.source\.js$/),
          type: 'asset/source',
          as: '[resource-path].txt',
          oneOf: [
            { test: condition(/\.js$/), use: use('wrong:') },
            { test: condition(/\.txt$/), use: use('nested:') },
          ],
        },
        {
          test: condition(/\.source\.js\.txt$/),
          include: condition(/[/\\]rule-as[/\\]/),
          use: use('next:'),
          as: '[resource-path].processed',
          rules: [
            {
              test: condition(/\.txt\.processed$/),
              use: use('child:'),
              as: '[resource-path].final',
            },
          ],
        },
        {
          test: condition(/\.processed\.final$/),
          use: use('final:'),
          as: '[resource-path].selected',
          oneOf: [{ test: condition(/never-match/), as: 'wrong.txt' }],
        },
        {
          test:
            mode === 'mixed'
              ? (value: string) =>
                  /\.source\.js\.txt\.processed\.final\.selected$/.test(value)
              : condition(/\.source\.js\.txt\.processed\.final\.selected$/),
          use: use('selected:'),
        },
        {
          test: condition(/\.final\.selected$/),
          resourceQuery: condition(/^\?inline$/),
          resourceFragment: condition(/^#fragment$/),
          use: use('query:'),
        },
        {
          test: condition(/literal\.source\.js$/),
          type: 'asset/source',
          as: 'virtual/#dir/[resourcePath]-[name]*?part.txt',
          oneOf: [
            {
              test: condition(/\.txt$/),
              as: '[resource-path].candidate',
              oneOf: [{ test: condition(/never-match/), as: 'wrong.txt' }],
            },
            {
              test: condition(/\.candidate$/),
              as: '[resource-path].chosen',
              use: use('choice:'),
            },
            { test: condition(/\.chosen$/), use: use('wrong:') },
          ],
        },
        {
          test: condition(
            /^virtual\/#dir\/\[resourcePath\]-\[name\]\*\?part\.txt\.candidate\.chosen$/,
          ),
          resourceQuery: condition(/^\?literal#value$/),
          resourceFragment: condition(/^#fragment$/),
          use: use('literal:'),
        },
      ],
    },
  });
});
