import fs from 'node:fs';
import path from 'node:path';
import type { Stats } from '../Stats';

let reportIndex = 0;
const processStartTimestamp = new Date().toISOString();

/** Sample RSS during one compilation without retaining per-sample data. */
export function startCompilationRssSampler(): () => number {
  let peakRssBytes = process.memoryUsage().rss;
  const sample = () => {
    peakRssBytes = Math.max(peakRssBytes, process.memoryUsage().rss);
  };
  const timer = setInterval(sample, 100);
  timer.unref();
  return () => {
    clearInterval(timer);
    sample();
    return peakRssBytes;
  };
}

/** Write one bounded, opt-in process and Rspack allocator snapshot after a build. */
export function reportCompilationMemory(
  directory: string,
  stats: Stats,
  allocator: 'default' | 'jemalloc',
  rustHeapAllocatedBytes: number | null,
  phase: 'build' | 'rebuild',
  compilationPeakRssBytes: number | null,
): void {
  try {
    const memory = process.memoryUsage();
    const resourceUsage = process.resourceUsage();
    const rssPeakUnit = 1024;
    const startTime = stats.startTime;
    const endTime = stats.endTime;
    const index = reportIndex++;
    const report = {
      schema_version: 1,
      phase,
      process_id: process.pid,
      process_start_timestamp: processStartTimestamp,
      timestamp: new Date().toISOString(),
      duration_ms:
        startTime !== undefined && endTime !== undefined
          ? Math.max(0, endTime - startTime)
          : null,
      allocator,
      rust_live_bytes: rustHeapAllocatedBytes,
      process: {
        rss_bytes: memory.rss,
        peak_rss_bytes:
          compilationPeakRssBytes ?? resourceUsage.maxRSS * rssPeakUnit,
        process_peak_rss_bytes: resourceUsage.maxRSS * rssPeakUnit,
        heap_used_bytes: memory.heapUsed,
        heap_total_bytes: memory.heapTotal,
        external_bytes: memory.external,
        array_buffers_bytes: memory.arrayBuffers,
      },
    };

    fs.mkdirSync(directory, { recursive: true });
    const file = path.join(
      directory,
      `rspack-memory-${process.pid}-${index}.json`,
    );
    fs.writeFileSync(file, `${JSON.stringify(report)}\n`, { flag: 'wx' });
  } catch (error) {
    // Diagnostics must not change the result of a compilation.
    console.warn('[rspack-memory] could not write memory report', error);
  }
}
