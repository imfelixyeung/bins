/**
 * Checks if the etag for the CSVs has changed
 */

import { db } from "@repo/database/src";
import { eq } from "@repo/database/src/orm";
import { etagsTable } from "@repo/database/src/schema";

const getEtagFromFetch = async (url: string) => {
  try {
    const response = await fetch(url, { method: "HEAD" });
    return response.headers.get("etag");
  } catch (error) {
    return null;
  }
};

const getEtagFromDb = async (url: string) => {
  const record = await db.query.etagsTable.findFirst({
    where: eq(etagsTable.url, url),
  });
  return record?.etag ?? null;
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
    await db.insert(etagsTable).values({ url, etag }).onConflictDoUpdate({
      target: etagsTable.url,
      set: { etag },
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
