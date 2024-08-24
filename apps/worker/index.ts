import { $, Result, ResultPromise } from "execa";
import schedule from "node-schedule";
import express from "express";
import PQueue from "p-queue";
import pino from "pino";
import pinoHttp from "pino-http";
import etag from "./etag";
import { urls } from "./data";

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

const returnEtagsIfNeedUpdate = async () => {
  const jobs = await etag.get(urls.jobs);
  const premises = await etag.get(urls.premises);

  // if latest etags are null, we cant determine if we need to update
  if (!jobs.latest || !premises.latest) {
    logger.warn("Unable to determine if we need to update");
    return null;
  }

  // if the etags are the same, we don't need to update
  if (
    jobs.latest.etag === jobs.database?.etag &&
    premises.latest.etag === premises.database?.etag
  ) {
    return null;
  }

  // if the etags are different, we need to update
  return { jobs, premises };
};

const run = async () => {
  logger.info("Running update script");

  const etags = await returnEtagsIfNeedUpdate();
  if (!etags) {
    logger.info("No need to update");
    return;
  }

  logger.info({ etags }, "Update required, etags updated");

  const start = new Date();
  status = {
    running: true,
  };

  let result: Result<{}> | null = null;

  try {
    // run the ./scripts/update.sh script
    result = await $`sh ./scripts/update.sh`;
    const { stdout, stderr, exitCode } = result;

    if (result.exitCode !== 0) {
      throw new Error("Exit code was not 0");
    }

    const end = new Date();
    const duration = end.getTime() - start.getTime();
    status = {
      running: false,
      lastRun: {
        start,
        end,
        duration,
        stdout,
        stderr,
        exitCode,
      },
    };

    logger.info({ start, end, duration }, "Finished update script");

    await Promise.allSettled([etag.set(etags.jobs), etag.set(etags.premises)]);
  } catch (error) {
    logger.error({ error }, "Error running update script");
    const end = new Date();
    status = {
      running: false,
      lastRun: {
        start,
        end,
        duration: end.getTime() - start.getTime(),
        stdout: result?.stdout ?? "",
        stderr: result?.stderr ?? "",
        exitCode: result?.exitCode,
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

app.use(pinoHttp());

app.post("/update", async (req, res) => {
  logger.info("Update triggered via API /update");
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
  logger.info("Status requested /status");
  res.json(status);
});

if (process.env.ONE_SHOT) {
  queueRun().then(() => process.exit());
} else {
  app.listen(3000, () => {
    console.log("Server listening on port 3000");
  });

  // every day at 3:00 AM
  schedule.scheduleJob("0 3 * * *", () => {
    logger.info("Triggering update via cron job");
    queueRun();
  });
}
