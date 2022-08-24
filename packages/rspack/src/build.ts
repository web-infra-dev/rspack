import { Rspack } from '.';
export async function build(config: any) {
  const rspack = new Rspack(config);
  return await rspack.build();
}
