import schema from './schema.json';
import { validate } from 'schema-utils';

const configuration = { name: 'RSpack raw options' };

export const validateRawOptions = (opts: any) => validate(schema as any, opts, configuration);
