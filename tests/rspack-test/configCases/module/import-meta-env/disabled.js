if (typeof import.meta.env === 'undefined') {
  import.meta.env = { RUNTIME: 'runtime' };
}

const { env } = import.meta;
const { AAA } = import.meta.env;

export default {
  direct: import.meta.env,
  destructured: env,
  defined: import.meta.env.AAA,
  destructuredDefined: AAA,
  definedType: typeof import.meta.env.AAA,
  type: typeof import.meta.env,
};
