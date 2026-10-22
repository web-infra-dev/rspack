import { defineConfig } from '@rspack/cli';

const nullValue = null;
const undefinedValue = undefined;
const falseValue = false;
const zeroValue = 0;
const emptyStringValue = '';

class FailPlugin {
  apply() {
    throw new Error('FailedPlugin');
  }
}

export default defineConfig({
  plugins: [
    undefinedValue && new FailPlugin(),
    nullValue && new FailPlugin(),
    falseValue && new FailPlugin(),
    zeroValue && new FailPlugin(),
    emptyStringValue && new FailPlugin(),
  ],
});
