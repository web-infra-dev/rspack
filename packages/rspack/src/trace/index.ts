// following chrome trace event format https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU/preview?tab=t.0#heading=h.uxpopqvbjezh
export interface ChromeEvent {
  name: string;
  trackName?: string;
  ph: 'b' | 'e' | 'X' | 'P';
  processName?: string;
  categories?: string[];
  uuid: number;
  ts: bigint;
  args?: {
    [key: string]: any;
  };
}
type MakeOptional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

type PartialChromeEvent = MakeOptional<ChromeEvent, 'ts' | 'ph'>;

// this is a tracer for nodejs
// FIXME: currently we only support chrome layer and do nothing for logger layer
export class JavaScriptTracer {
  static state: 'uninitialized' | 'on' | 'off' = 'uninitialized';
  // baseline time, we use offset time for tracing to align with rust side time
  static startTime: bigint;
  static events: ChromeEvent[];
  static layer: string;
  // tracing file path, same as rust tracing-chrome's
  static output: string;
  // inspector session for CPU Profiler
  static session: import('node:inspector').Session;
  // plugin counter for different channel in trace viewer, choose 100 to avoid conflict with known tracks
  private static counter = 10000;

  /**
   * only first call take effects, subsequent calls will be ignored
   * @param layer tracing layer
   * @param output tracing output file path
   */
  static async initJavaScriptTrace(layer: string, output: string) {
    const { Session } = await import('node:inspector');
    this.session = new Session();
    this.layer = layer;
    this.output = output;
    this.events = [];
    this.state = 'on';
    this.startTime = process.hrtime.bigint(); // use microseconds
  }

  static uuid() {
    return this.counter++;
  }
  static initCpuProfiler() {
    if (this.layer) {
      this.session.connect();
      this.session.post('Profiler.enable');
      this.session.post('Profiler.start');
    }
  }
  /**
   * only first call take effects, subsequent calls will be ignored
   * @param isEnd true means we are at the end of tracing,and can append ']' to close the json
   * @returns
   */
  static async cleanupJavaScriptTrace() {
    if (this.state === 'uninitialized') {
      throw new Error(
        'JavaScriptTracer is not initialized, please call initJavaScriptTrace first',
      );
    }
    if (!this.layer || this.state === 'off') {
      return;
    }
    const profileHandler = (
      err: Error | null,
      param: import('node:inspector').Profiler.StopReturnType,
    ) => {
      let cpu_profile: import('node:inspector').Profiler.Profile | undefined;
      if (err) {
        console.error('Error stopping profiler:', err);
      } else {
        cpu_profile = param.profile;
      }
      if (cpu_profile) {
        const uuid = this.uuid();
        // add event contains cpu_profile to show cpu profile in trace viewer (firefox profiler and perfetto)
        // more info in https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU/preview?tab=t.0#heading=h.yr4qxyxotyw
        this.pushEvent({
          name: 'Profile',
          ph: 'P',
          trackName: 'JavaScript CPU Profiler',
          processName: 'JavaScript CPU',
          uuid,
          ...this.getCommonEv(),
          categories: ['disabled-by-default-v8.cpu_profiler'],
          args: {
            data: {
              startTime: 0, // use offset time to align with other trace data
            },
          },
        });

        this.pushEvent({
          name: 'ProfileChunk',
          ph: 'P',
          trackName: 'JavaScript CPU Profiler',
          processName: 'JavaScript CPU',
          ...this.getCommonEv(),
          categories: ['disabled-by-default-v8.cpu_profiler'],
          uuid,
          args: {
            data: {
              cpuProfile: cpu_profile,
              timeDeltas: cpu_profile.timeDeltas,
            },
          },
        });
      }
    };
    await new Promise<void>((resolve, reject) => {
      this.session.post('Profiler.stop', (err, params) => {
        if (err) {
          reject(err);
        } else {
          try {
            profileHandler(err, params);
            resolve();
          } catch (err) {
            reject(err);
          }
        }
      });
    });
    this.state = 'off';
  }
  // get elapsed time since start(nanoseconds same as rust side timestamp)
  static getTs() {
    const now: bigint = process.hrtime.bigint();
    const elapsed = now - this.startTime;
    return elapsed;
  }
  // get common chrome event
  static getCommonEv() {
    return {
      ts: this.getTs(),
      cat: 'rspack',
    };
  }
  static pushEvent(event: ChromeEvent) {
    const stringifiedArgs = Object.keys(event.args || {}).reduce<
      Record<string, string>
    >((acc, key) => {
      acc[key] = JSON.stringify(event.args![key]);
      return acc;
    }, {});
    this.events.push({
      ...event,
      args: stringifiedArgs,
    });
  }
  // start an chrome async event
  static startAsync(events: PartialChromeEvent) {
    if (!this.layer) {
      return;
    }
    this.pushEvent({
      ...this.getCommonEv(),
      ...events,
      ph: 'b',
    });
  }
  // end an chrome async event
  static endAsync(events: PartialChromeEvent) {
    if (!this.layer) {
      return;
    }
    this.pushEvent({
      ...this.getCommonEv(),
      ...events,
      ph: 'e',
    });
  }
}
