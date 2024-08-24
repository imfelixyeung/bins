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
        `Attempt ${error.attemptNumber} failed. There are ${error.retriesLeft} retries left. (${error.message})`
      );
    },
    retries: 3,
  }).catch(() => null);
  const etagLatest = await pRetry(() => getEtagFromFetch(url), {
    onFailedAttempt: (error) => {
      logger.warn(
        `Attempt ${error.attemptNumber} failed. There are ${error.retriesLeft} retries left. (${error.message})`
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
  try {
    await db
      .insert(etagsTable)
      .values({ url, etag: etag?.etag, updatedAt: etag?.lastModified })
      .onConflictDoUpdate({
        target: etagsTable.url,
        set: { etag: etag?.etag, updatedAt: etag?.lastModified },
      });
  } catch (error) {
    return;
  }
};

const etag = {
  get: checkEtag,
  set: setEtag,
};

export default etag;
