import cors from "cors";
import type { Express } from "express";
import express from "express";
import { premisesRouter } from "./routes/premises.route";

const PORT = 80;

export const app: Express = express();
app.use(cors());

app.use("/premises", premisesRouter);

export const start = (app: Express) =>
  app.listen(PORT, () => {
    console.log(`Server listening on port ${PORT}`);
  });
