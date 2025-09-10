import { fetch } from "undici";
import { urls } from "../data";
import etag from "../etag";
import TextLineStream from "textlinestream";
import { db } from "@repo/database/src";
import { sql } from "@repo/database/src/orm";
import { integer, pgTable, text, timestamp } from "@repo/database/src/orm-pg";
import { premisesTable } from "@repo/database/src/schema";
import { nullIfEmpty } from "../utils";

const ROW_BATCH_INSERT = 5000;

const stagingPremisesTable = pgTable("staging_dm_premises", {
  id: integer("id").primaryKey(),
  addressRoom: text("address_room"),
  addressNumber: text("address_number"),
  addressStreet: text("address_street"),
  addressLocality: text("address_locality"),
  addressCity: text("address_city"),
  addressPostcode: text("address_postcode"),
  searchPostcode: text("search_postcode"),
  createdAt: timestamp("created_at", { withTimezone: true })
    .notNull()
    .defaultNow(),
  updatedAt: timestamp("updated_at", { withTimezone: true })
    .notNull()
    .defaultNow(),
});

export const updatePremises = async () =>
  etag.run(urls.premises, async (url) => {
    const now = new Date();
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
    await db.execute(sql`DROP TABLE IF EXISTS ${stagingPremisesTable};`);

    // create temporary tables to hold the data
    await db.execute(
      sql`CREATE TEMP TABLE ${stagingPremisesTable} AS TABLE ${premisesTable} WITH NO DATA;`
    );

    const rows: string[] = [];

    const commitRows = async () => {
      console.log("committing", rows.length);

      const values: (typeof stagingPremisesTable.$inferInsert)[] = rows.map(
        (row) => {
          const line = row.split(",");
          const [
            id,
            addressRoom,
            addressNumber,
            addressStreet,
            addressLocality,
            addressCity,
            addressPostcode,
          ] = line;

          return {
            id: Number(id),
            addressRoom: nullIfEmpty(addressRoom),
            addressNumber: nullIfEmpty(addressNumber),
            addressStreet: nullIfEmpty(addressStreet),
            addressLocality: nullIfEmpty(addressLocality),
            addressCity: nullIfEmpty(addressCity),
            addressPostcode: nullIfEmpty(addressPostcode),
            createdAt: now,
            updatedAt: now,
          };
        }
      );

      await db.insert(stagingPremisesTable).values(values);

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

    // copy address_postcode column to search_postcode column, removing spaces, and converting to upper case
    await db.execute(
      sql`UPDATE ${stagingPremisesTable} SET search_postcode = upper(replace(address_postcode, ' ', ''));`
    );

    await db.transaction(async (tx) => {
      const count = await tx.$count(stagingPremisesTable);
      if (count === 0) return;

      await tx.execute(sql`DELETE FROM ${premisesTable};`);
      await tx.execute(sql`
        INSERT INTO ${premisesTable}
          (id, address_room, address_number, address_street, address_locality,  address_city, address_postcode, search_postcode)
        SELECT
          id, address_room, address_number, address_street, address_locality,   address_city, address_postcode, search_postcode
        FROM ${stagingPremisesTable};
      `);
    });

    return;
  });
