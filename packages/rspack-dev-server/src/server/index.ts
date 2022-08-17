import express from 'express';

export async function createServer(options): Promise<express.Express> {
  const {
    dev: {
      static: {
        directory = 'dist'
      } = {}
    } = {}
  } = options || {};

  const app = express();
  
  app.use(express.static(directory));

  return app;
}

