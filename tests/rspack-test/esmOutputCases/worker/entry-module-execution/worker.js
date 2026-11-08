#!/usr/bin/env node
'use client';

export default 'worker';

globalThis.__workerEntryExecuted = true;
