import { $, Result, ResultPromise } from "execa";
import schedule from "node-schedule";
import express from "express";
import PQueue from "p-queue";
import pinoHttp from "pino-http";
import etag from "./etag";
import { urls } from "./data";
import logger from "./logger";
import { updatePremises } from "./update/premises";
import { updateJobs } from "./update/jobs";
import { vacuumFull } from "./db";

const queue = new PQueue({ concurrency: 1 });

type Status = {
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
};

let status: {
  jobs: Status;
  premises: Status;
} = {
  jobs: {
    running: false,
  },
  premises: {
    running: false,
  },
};

const updateDataset = async (dataset: "jobs" | "premises") => {
  const datasetLogger = logger.child({ dataset });
  const url = urls[dataset];
  const urlEtag = await etag.get(url);

  datasetLogger.info({ urlEtag }, "Checking if dataset needs updating");

  if (!urlEtag.latest) {
    datasetLogger.warn("Unable to determine if we need to update");
    return null;
  }

  if (urlEtag.latest.etag === urlEtag.database?.etag) {
    datasetLogger.info("Dataset unchanged, no need to update");
    return null;
  }

  datasetLogger.info("Dataset updated, updating");

  const start = new Date();
  let result: Result<{}> | null = null;

  try {
    // run the ./scripts/update.sh script
    result = await $`sh ./scripts/update-${dataset}.sh`;
    const { stdout, stderr, exitCode } = result;

    if (result.exitCode !== 0) {
      throw new Error("Exit code was not 0");
    }

    const end = new Date();
    const duration = end.getTime() - start.getTime();
    status[dataset] = {
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

    datasetLogger.info({ start, end, duration }, "Finished update script");

    await etag.set(urlEtag);
  } catch (error) {
    datasetLogger.error({ error }, "Error running update script");
    const end = new Date();
    status[dataset] = {
      running: false,
      lastRun: {
        start,
        end,
        duration: end.getTime() - start.getTime(),
        stdout: result?.stdout ?? "",
        stderr: result?.stderr ?? "",
        exitCode: result?.exitCode,
        error: {
          message: error instanceof Error ? error.message : String(error),
        },
      },
    };
  }
};

const run = async () => {
  logger.info("Run triggered");

  const premisesUpdated = await updatePremises().catch(() => null);
  const jobsUpdated = await updateJobs().catch(() => null);

  if (premisesUpdated || jobsUpdated) {
    await vacuumFull().catch(() => null);
  }

  logger.info("Finished run");
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
  queueRun()
    .then(async () => console.log(status))
    .then(() => process.exit());
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
