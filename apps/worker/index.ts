import { $ } from "execa";
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
    error?: {
      message: string;
    };
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

  try {
    // run the ./scripts/update.sh script
    const { stdout, stderr, exitCode } = await $`sh ./scripts/update.sh`;
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
  } catch (error) {
    const end = new Date();
    status = {
      running: false,
      lastRun: {
        start,
        end,
        duration: end.getTime() - start.getTime(),
        stdout: "",
        stderr: "",
        error: {
          message: error instanceof Error ? error.message : error,
        },
      },
    };
  }
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
  try {
    queueRun();
  } catch (error) {
    res.status(500).json({
      error: true,
      message: "Error triggering update",
    });
    return;
  }

  res.json({
    success: true,
    message: "Update triggered",
  });
});

app.get("/status", async (req, res) => {
  res.json(status);
});

app.listen(3000, () => {
  console.log("Server listening on port 3000");
});

// every day at 2:00 AM
schedule.scheduleJob("0 2 * * *", queueRun);
