import rspack, { Configuration, Stats, StatsError } from "@rspack/core";

const errorFromStats = (stats: Stats | undefined): Error => {
  const errors = stats?.toJson()?.errors;
  if (!errors) {
    return new Error("No stats");
  }
  return new Error(
    "Error:" + errors.map((error: StatsError) => error.message).join(", ")
  );
};

export const runRspack = (options: Configuration): Promise<Stats> =>
  new Promise((resolve, reject) => {
    rspack(options, (err: Error | null, stats: Stats | undefined) => {
      if (err) {
        reject(err);
      } else if (stats?.hasErrors() === false) {
        resolve(stats);
      } else {
        reject(errorFromStats(stats));
      }
    });
  });
