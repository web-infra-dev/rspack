name: 'CodSpeed Performance Analysis'
description: 'Continuous benchmarking and performance checks'
branding:
  color: orange
  icon: activity

author: 'Arthur Pastel'
inputs:
  run:
    description: 'The command to run the benchmarks'
    required: true

  mode:
    description: |
      The mode to run the benchmarks in. The following modes are available:
      - `simulation`: Run the benchmarks with CPU simulation measurements.
      - `walltime`: Run the benchmarks with walltime measurement.
      - `memory`: Run the benchmarks with allocation measurements.
      - `instrumentation`: (Deprecated) Legacy name for `simulation`. Please use `simulation` instead.

      We strongly recommend starting with the `simulation` mode.

      Using the `walltime` mode on traditional VMs/Hosted Runners might lead to inconsistent data. For the best results, we recommend using CodSpeed Hosted Macro Runners, which are fine-tuned for performance measurement consistency.
      Check out the [Walltime Instrument Documentation](https://docs.codspeed.io/instruments/walltime/) for more details.
    required: true

  token:
    description: |
      CodSpeed upload token. Only required for private repositories.
    required: false

  working-directory:
    description: |
      The directory where the `run` command will be executed.
      Warning: if you use defaults.working-directory, you must still set this parameter.
    required: false

  upload-url:
    description: 'The upload endpoint (for on-premise deployments)'
    required: false

  runner-version:
    description: "The version of the runner to use. Use 'latest' to automatically fetch the latest release version from GitHub, or specify a version like '3.5.0' or 'v3.5.0'."
    required: false

  instruments:
    description: |
      Comma separated list of instruments to enable. The following instruments are available:
      - `mongodb`: MongoDB instrumentation, requires the MongoDB instrument to be enabled for the organization in CodSpeed
    required: false

  mongo-uri-env-name:
    description: |
      The name of the environment variable containing the MongoDB URI. Requires the `mongodb` instrument to be activated in `instruments`.
      If the instrumentation is enabled and this value is not set, the user will need to dynamically provide the MongoDB URI to the CodSpeed runner.
    required: false

  cache-instruments:
    description: |
      Enable caching of instrument installations (like valgrind or perf) to speed up subsequent workflow runs. Set to 'false' to disable caching.
    required: false
    default: 'true'

  instruments-cache-dir:
    description: |
      The directory to use for caching installations of instruments (like valgrind or perf). Defaults to `$HOME/.cache/codspeed-action`.
    required: false
    default: '~/.cache/codspeed-action'

  allow-empty:
    description: |
      Allow the action to complete successfully even if no benchmarks were found or run. Set to 'true' to enable this behavior.
    required: false
    default: 'false'

runs:
  using: 'composite'
  steps:
    - shell: bash
      run: |
        # Validate required inputs
        # (custom message for smoother v4 migration)
        if [ -z "${{ inputs.mode }}" ]; then
          echo "::error title=Missing required input 'mode'::The 'mode' input is required as of CodSpeed Action v4. Please explicitly set 'mode' to 'simulation' or 'walltime'. Before, this variable was automatically set to instrumentation on every runner except for CodSpeed macro runners where it was set to walltime by default. See https://codspeed.io/docs/instruments for details."
          exit 1
        fi

        # We can use official runner if it supports config valgrind flags in the future: https://github.com/CodSpeedHQ/runner/pull/92
        cargo install --git https://github.com/CPunisher/runner.git --rev 9c1ca5aa4742b8524843c0ac3e417c6ecb91b1bd codspeed-runner

        # Get the runner arguments
        RUNNER_ARGS=""
        if [ -n "${{ inputs.token }}" ]; then
          RUNNER_ARGS="$RUNNER_ARGS --token ${{ inputs.token }}"
        fi
        if [ -n "${{ inputs.working-directory }}" ]; then
          RUNNER_ARGS="$RUNNER_ARGS --working-directory=${{ inputs.working-directory }}"
        fi
        if [ -n "${{ inputs.upload-url }}" ]; then
          RUNNER_ARGS="$RUNNER_ARGS --upload-url=${{ inputs.upload-url }}"
        fi
        if [ -n "${{ inputs.mode }}" ]; then
          RUNNER_ARGS="$RUNNER_ARGS --mode=${{ inputs.mode }}"
        fi
        if [ -n "${{ inputs.instruments }}" ]; then
          RUNNER_ARGS="$RUNNER_ARGS --instruments=${{ inputs.instruments }}"
        fi
        if [ -n "${{ inputs.mongo-uri-env-name }}" ]; then
          RUNNER_ARGS="$RUNNER_ARGS --mongo-uri-env-name=${{ inputs.mongo-uri-env-name }}"
        fi
        if [ "${{ inputs.cache-instruments }}" = "true" ] && [ -n "${{ inputs.instruments-cache-dir }}" ]; then
          RUNNER_ARGS="$RUNNER_ARGS --setup-cache-dir=${{ inputs.instruments-cache-dir }}"
        fi
        if [ "${{ inputs.allow-empty }}" = "true" ]; then
          RUNNER_ARGS="$RUNNER_ARGS --allow-empty"
        fi

        # Run the benchmarks
        # Enable fair sched to make benchmark more stable, see: https://github.com/CodSpeedHQ/runner/pull/91
        env VALGRIND_FLAGS='--fair-sched=yes' codspeed run $RUNNER_ARGS -- '${{ inputs.run }}'