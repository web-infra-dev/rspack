const direct = import.meta.rstest;
const optional = import.meta.rstest?.source;
const type = typeof import.meta.rstest;
let branch = false;
if (import.meta.rstest) {
  branch = true;
}

export {
  branch,
  direct,
  optional,
  type,
};
