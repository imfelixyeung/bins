import { relations } from "drizzle-orm";
import {
  integer,
  pgTable,
  text,
  date,
  timestamp,
  serial,
  index,
} from "drizzle-orm/pg-core";

export const premisesTable = pgTable(
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

    createdAt: timestamp("created_at", { withTimezone: true })
      .notNull()
      .defaultNow(),
    updatedAt: timestamp("updated_at", { withTimezone: true })
      .notNull()
      .defaultNow(),
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

export const jobsTable = pgTable(
  "dm_jobs",
  {
    id: serial("id").primaryKey(),

    premisesId: integer("premises_id").notNull(),
    bin: text("bin").notNull(),
    date: date("date").notNull(),
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

export const etagsTable = pgTable("etags", {
  id: serial("id").primaryKey(),
  url: text("url").notNull().unique(),
  etag: text("etag"),
  createdAt: timestamp("created_at", { withTimezone: true })
    .notNull()
    .defaultNow(),
  updatedAt: timestamp("updated_at", { withTimezone: true })
    .notNull()
    .defaultNow(),
});
