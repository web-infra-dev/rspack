export const createAdd = async () => {
  const addModule = await import.source("./add.wat");
  return new WebAssembly.Instance(addModule).exports.add;
};
