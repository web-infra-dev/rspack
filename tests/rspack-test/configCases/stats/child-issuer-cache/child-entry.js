import { first } from './first';
import { second } from './second';

if (first() + second() !== 42) throw new Error('Unexpected child module values');
