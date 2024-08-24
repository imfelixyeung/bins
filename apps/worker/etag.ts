/**
 * Checks if the etag for the CSVs has changed
 */

import { db } from "@repo/database/src";
import { eq } from "@repo/database/src/orm";
import { etagsTable } from "@repo/database/src/schema";

const getEtagFromFetch = async (url: string) => {
  try {
    const response = await fetch(url, { method: "HEAD" });

    const etag = response.headers.get("etag");
    const lastModified = response.headers.get("last-modified");

    if (!etag || !lastModified) {
      return null;
    }

    return {
      etag,
      lastModified: new Date(lastModified),
    };
  } catch (error) {
    return null;
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
  const etagDatabase = await getEtagFromDb(url);
  const etagLatest = await getEtagFromFetch(url);

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
