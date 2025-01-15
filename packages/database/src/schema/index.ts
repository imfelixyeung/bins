import { relations } from "drizzle-orm";
import { integer, sqliteTable, text, index } from "drizzle-orm/sqlite-core";

export const premisesTable = sqliteTable(
  "dm_premises",
  {
    id: integer("id").primaryKey(),
    addressRoom: text("address_room"),
    addressNumber: text("address_number"),
    addressStreet: text("address_street"),
    addressLocality: text("address_locality"),
    addressCity: text("address_city"),
    addressPostcode: text("address_postcode"),

    // used for searching
    searchPostcode: text("search_postcode"),

    createdAt: text("created_at")
      .$defaultFn(() => new Date().toISOString())
      .notNull(),
    updatedAt: integer("updated_at", { mode: "timestamp" }).$onUpdate(
      () => new Date()
    ),
  },
  (table) => ({
    searchPostcodeIndex: index("search_postcode_index").on(
      table.searchPostcode
    ),
  })
);

export const premisesRelation = relations(premisesTable, ({ many }) => ({
  jobs: many(jobsTable),
}));

export const jobsTable = sqliteTable(
  "dm_jobs",
  {
    id: integer("id").primaryKey(),

    premisesId: integer("premises_id").notNull(),
    bin: text("bin").notNull(),
    date: text("date").notNull(),
  },
  (table) => ({
    premisesIdIndex: index("jobs_premises_id_index").on(table.premisesId),
  })
);

export const jobsRelation = relations(jobsTable, ({ one }) => ({
  premises: one(premisesTable, {
    fields: [jobsTable.premisesId],
    references: [premisesTable.id],
  }),
}));

export const etagsTable = sqliteTable("etags", {
  id: integer("id").primaryKey(),
  url: text("url").notNull().unique(),
  etag: text("etag"),
  createdAt: text("created_at")
    .$defaultFn(() => new Date().toISOString())
    .notNull(),
  updatedAt: integer("updated_at", { mode: "timestamp" }).$onUpdate(
    () => new Date()
  ),
});
