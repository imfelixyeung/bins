import { relations } from "drizzle-orm";
import { bigserial, date, pgTable, text } from "drizzle-orm/pg-core";

export const premises = pgTable("premises", {
  id: text("id").primaryKey(),
  addressRoom: text("address_room"),
  addressHouseNumber: text("address_house_number"),
  addressStreet: text("address_street"),
  addressLocality: text("address_locality"),
  addressCity: text("address_city"),
  addressPostCode: text("address_post_code").notNull(),
});

export const collections = pgTable("collections", {
  id: bigserial("id", { mode: "bigint" }).primaryKey(),
  premiseId: text("premise_id").notNull(),
  binColour: text("bin_colour").notNull(),
  date: date("date").notNull(),
});

export const premisesRelations = relations(premises, ({ many }) => ({
  collections: many(collections),
}));

export const collectionsRelations = relations(collections, ({ one }) => ({
  premise: one(premises, {
    fields: [collections.premiseId],
    references: [premises.id],
  }),
}));