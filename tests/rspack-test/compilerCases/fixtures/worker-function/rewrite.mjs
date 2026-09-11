export default async (data, options) => {
  if (data.request === './virtual') data.request = options.other;
};
