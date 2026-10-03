// Loaded only into the benchmark process, after exec and before Rust main.
// pthreads default to DEFAULT QoS even when main is USER_INITIATED. Set each
// new thread explicitly, preserving the caller's attributes and start routine.
#include <errno.h>
#include <pthread.h>
#include <pthread/qos.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void set_benchmark_qos(void) {
  int error = pthread_set_qos_class_self_np(QOS_CLASS_USER_INITIATED, 0);
  if (error) {
    fprintf(stderr, "pthread_set_qos_class_self_np: %s\n", strerror(error));
    _Exit(1);
  }
}

__attribute__((constructor)) static void benchmark_qos(void) {
  set_benchmark_qos();
  fputs("bench-qos:user-initiated\n", stderr);
}

struct start_context {
  void *(*routine)(void *);
  void *argument;
};

static void *start_with_qos(void *opaque) {
  struct start_context context = *(struct start_context *)opaque;
  free(opaque);
  set_benchmark_qos();
  return context.routine(context.argument);
}

static int create_with_qos(pthread_t *thread, const pthread_attr_t *attributes,
                           void *(*routine)(void *), void *argument) {
  static _Thread_local int creating;
  if (creating) {
    fputs("recursive pthread_create interposition\n", stderr);
    _Exit(1);
  }
  struct start_context *context = malloc(sizeof(*context));
  if (!context) return ENOMEM;
  context->routine = routine;
  context->argument = argument;
  // Calls from the interposing image bind to the original implementation.
  creating = 1;
  int error = pthread_create(thread, attributes, start_with_qos, context);
  creating = 0;
  if (error) free(context);
  return error;
}

__attribute__((used, section("__DATA,__interpose")))
static const struct { const void *replacement; const void *original; } interpose = {
  (const void *)create_with_qos, (const void *)pthread_create
};
