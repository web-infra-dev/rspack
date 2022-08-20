import type { RspackOptions } from "./config";
import Ajv from 'ajv';
import betterAjvErrors from 'better-ajv-errors';

const validator = new Ajv({
  strict: false,
  allErrors: true,
  $data: true,
});


export function validate(schema: Record<string, string>, options: RspackOptions): boolean {
  const compiledSchema = validator.compile(schema);
  if (compiledSchema(options)) {
    return true;
  } else if (compiledSchema.errors) {
    throw Error(betterAjvErrors(schema, options, compiledSchema.errors, {
      indent: 2,
    }));
  }
  return true;
}