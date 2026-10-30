import { parentPort } from 'node:worker_threads';
import shared from './shared.cjs';

parentPort.postMessage(shared.value);
