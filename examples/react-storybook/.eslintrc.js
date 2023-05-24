const ignore = 0;

module.exports = {
  overrides: [
    {
      files: './src/**/*',
      env: {
        browser: true,
      },
      rules: {
        'react/no-this-in-sfc': ignore,
        'import/no-unresolved': ignore,
        'react/react-in-jsx-scope': ignore,
        'import/no-extraneous-dependencies': ignore,
        'global-require': ignore,
        'no-redeclare': ignore,
        'react/prop-types': ignore,
      },
    },
  ],
};
