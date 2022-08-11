import express from 'express';
import { Rspack } from "@rspack/core";

export async function createServer(options): Promise<express.Express> {
  const { dev: {
    static : {
      directory = 'dist'
    } = {},
  } = {}} =  options || {}


  const rspack = new Rspack(options);
  await rspack.build();

  const app = express();


  app.use(express.static(directory));

  return app;
}

