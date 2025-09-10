import { fetch } from "undici";
import { urls } from "../data";
import etag from "../etag";
import TextLineStream from "textlinestream";
import { db } from "@repo/database/src";
import { sql } from "@repo/database/src/orm";
import {
  date,
  integer,
  pgTable,
  serial,
  text,
} from "@repo/database/src/orm-pg";
import { jobsTable } from "@repo/database/src/schema";

const ROW_BATCH_INSERT = 10000;

const stagingJobsTable = pgTable("staging_dm_jobs", {
  id: serial("id").primaryKey(),
  premisesId: integer("premises_id").notNull(),
  bin: text("bin").notNull(),
  date: date("date").notNull(),
});

export const updateJobs = async () =>
  etag.run(urls.jobs, async (url) => {
    const reader = await fetch(url).then((res) => {
      const body = res.body as ReadableStream<Uint8Array> | null;
      if (!body) return null;

      const reader = body
        // @ts-ignore
        .pipeThrough(new TextDecoderStream())
        .pipeThrough(new TextLineStream())
        .getReader() as ReadableStreamDefaultReader<string>;
      return reader;
    });
    if (!reader) throw new Error("Unable to obtain reader");

    // update date format to match source data
    await db.execute(sql`SET DateStyle TO 'ISO', 'DMY';`);

    // drop temporary tables just in case
    await db.execute(sql`DROP TABLE IF EXISTS ${stagingJobsTable};`);

    // create temporary tables to hold the data
    await db.execute(
      sql`CREATE TEMP TABLE ${stagingJobsTable} AS TABLE ${jobsTable} WITH NO DATA;`
    );

    const rows: string[] = [];

    const commitRows = async () => {
      console.log("committing", rows.length);

      const values: (typeof stagingJobsTable.$inferInsert)[] = rows.map(
        (row) => {
          const line = row.split(",");
          const [premisesId, bin, date] = line;

          return {
            premisesId: Number(premisesId),
            bin,
            date,
          };
        }
      );

      await db.insert(stagingJobsTable).values(values);

      rows.splice(0, rows.length);
    };

    await reader.read().then(async function pump({
      done,
      value,
    }): Promise<unknown> {
      if (done) return;
      rows.push(value);

      if (rows.length === ROW_BATCH_INSERT) {
        await commitRows();
      }

      return reader.read().then(pump);
    });

    await commitRows();

    await db.transaction(async (tx) => {
      const count = await tx.$count(stagingJobsTable);
      if (count === 0) return;

      await tx.execute(sql`DELETE FROM ${jobsTable};`);
      await tx.execute(sql`
        INSERT INTO ${jobsTable}
          (premises_id, bin, date)
        SELECT
          premises_id, bin, date
        FROM ${stagingJobsTable};
      `);
    });

    return;
  });
