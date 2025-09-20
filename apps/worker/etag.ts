/**
 * Checks if the etag for the CSVs has changed
 */

import { db } from "@repo/database/src";
import { eq } from "@repo/database/src/orm";
import { etagsTable } from "@repo/database/src/schema";
import { fetch, Agent } from "undici";
import pRetry from "p-retry";
import logger from "./logger";

const getEtagFromFetch = async (url: string) => {
  try {
    const now = new Date();
    const response = await fetch(url, {
      method: "HEAD",
      dispatcher: new Agent({ connect: { timeout: 60_000 } }),
    });

    if (!response.ok) {
      throw new Error("Response was not ok");
    }

    const etag = response.headers.get("etag");
    const lastModified = response.headers.get("last-modified");

    if (!etag || !lastModified) {
      throw new Error("Etag or last modified not found");
    }

    return {
      etag,
      lastModified: new Date(lastModified),
      checkedAt: now,
    };
  } catch (error) {
    throw error;
  }
};

const getEtagFromDb = async (url: string) => {
  const record = await db.query.etagsTable.findFirst({
    where: eq(etagsTable.url, url),
  });

  if (!record) {
    return null;
  }

  const { etag, updatedAt } = record;

  return {
    etag,
    lastModified: updatedAt,
  };
};

const checkEtag = async (url: string) => {
  const etagDatabase = await pRetry(() => getEtagFromDb(url), {
    onFailedAttempt: (error) => {
      logger.warn(
        `Attempt ${error.attemptNumber} failed. There are ${error.retriesLeft} retries left. (${error.error.message})`
      );
    },
    retries: 3,
  }).catch(() => null);
  const etagLatest = await pRetry(() => getEtagFromFetch(url), {
    onFailedAttempt: (error) => {
      logger.warn(
        `Attempt ${error.attemptNumber} failed. There are ${error.retriesLeft} retries left. (${error.error.message})`
      );
    },
    retries: 3,
  }).catch(() => null);

  return {
    url,
    database: etagDatabase,
    latest: etagLatest,
  };
};

type EtagResult = Awaited<ReturnType<typeof checkEtag>>;

const setEtag = async (data: EtagResult) => {
  const { url, latest: etag } = data;

  if (!etag) {
    throw new Error("Expected latest etag to be set");
  }

  try {
    await db
      .insert(etagsTable)
      .values({
        url,
        etag: etag.etag,
        updatedAt: etag.lastModified,
        checkedAt: etag.checkedAt,
      })
      .onConflictDoUpdate({
        target: etagsTable.url,
        set: {
          etag: etag.etag,
          updatedAt: etag.lastModified,
          checkedAt: etag.checkedAt,
        },
      });
  } catch (error) {
    return;
  }
};

/**
 * executes the function if the url etag has changed, otherwise do nothing
 *
 * @param url
 * @param func
 * @returns
 */
const runEtag = async <
  URL extends string,
  Func extends (url: URL) => Promise<any>,
>(
  url: URL,
  func: Func
): Promise<Awaited<ReturnType<Func>> | null> => {
  const etagLogger = logger.child({ name: "etag-runner" });

  etagLogger.info({ url }, "Checking etag has changed");
  const etag = await checkEtag(url);

  if (!etag.latest) {
    etagLogger.warn("Unable to determine if we need to update");
    return null;
  }

  if (etag.latest.etag === etag.database?.etag) {
    etagLogger.info("Etag unchanged, no need to update");
    await setEtag(etag);
    return null;
  }

  etagLogger.info("Etag updated, updating");

  const result = await func(url).catch((error) =>
    error instanceof Error ? error : new Error(error)
  );

  if (result instanceof Error) {
    etagLogger.error(result, "Error running function");
    throw result;
  }

  etagLogger.info("Function ran successfully, committing etag");

  await setEtag(etag);
  return result;
};

const etag = {
  get: checkEtag,
  set: setEtag,
  run: runEtag,
};

export default etag;
