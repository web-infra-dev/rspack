import { used } from 'prefix/sub';
import { deepUsed } from 'prefix/deep/sub';
import { exactUsed } from 'prefix/exact';
import { manual } from 'exact-directory';
import { used as exactFileUsed } from 'exact-file';

import { disabled } from 'prefix/disabled';
import { disabledSub } from 'prefix/disabled/sub';
import { overlapUsed } from 'overlap/deep/long-sub';

export default [used, deepUsed, exactUsed, manual, exactFileUsed, disabled, disabledSub, overlapUsed];
