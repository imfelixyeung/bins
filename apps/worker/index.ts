import { execa } from "execa";
import schedule from "node-schedule";
import express from "express";
import PQueue from "p-queue";
import pino from "pino";

const logger = pino();

const queue = new PQueue({ concurrency: 1 });

let status: {
  running: boolean;
  lastRun?: {
    start: Date;
    end: Date;
    duration: number;
    stdout: string;
    stderr: string;
    exitCode?: number | undefined;
  };
} = {
  running: false,
};

const run = async () => {
  logger.info("Running update script");
  const start = new Date();
  status = {
    running: true,
  };

  // run the ./scripts/update.sh script
  const { stdout, stderr, exitCode } = await execa`./scripts/update.sh`;

  const end = new Date();
  status = {
    running: false,
    lastRun: {
      start,
      end,
      duration: end.getTime() - start.getTime(),
      stdout,
      stderr,
      exitCode,
    },
  };

  logger.info("Finished update script");
};

const queueRun = async () => {
  // if queue size is greater than 1, don't run
  if (queue.size >= 1) return;

  await queue.add(run);
};

// create an express server on 3000 so that we can trigger the update manually
// also allow the status to be queried
const app = express();
app.post("/update", async (req, res) => {
  queueRun();
  res.send("OK");
});

app.get("/status", async (req, res) => {
  res.json(status);
});

app.listen(3000, () => {
  console.log("Server listening on port 3000");
});

// every day at 2:00 AM
schedule.scheduleJob("0 2 * * *", queueRun);
