import { db } from "@/db";
import { collections, premises } from "@/db/schema";
import { getHouseholdPremises } from "@/lib/leeds-opendata/household-waste-collections/household-premises";
import _ from "lodash";
import { getWasteCollections } from "./lib/leeds-opendata/household-waste-collections/waste-collections";

const transformDate = (date: string) => {
  // from 18/12/23 (DD/MM/YY) to 2018-12-23 (YYYY-MM-DD)
  const [day, month, year] = date.split("/");
  return `20${year}-${month}-${day}`;
};

export const populatePremises = async () => {
  const data = await getHouseholdPremises();

  if (data instanceof Error) {
    return data;
  }

  // split into chunks of 1000
  const chunks = _.chunk(data, 1000);

  const result = await Promise.all(
    chunks.map((chunk) =>
      db
        .insert(premises)
        .values(
          chunk.map((premise) => ({
            id: premise.premiseId,
            ...premise,
          }))
        )
        .onConflictDoNothing()
    )
  );
};

export const populateCollections = async () => {
  const data = await getWasteCollections();

  if (data instanceof Error) {
    return data;
  }

  data.forEach((row) => (row.date = transformDate(row.date)));

  // split into chunks of 1000
  const chunks = _.chunk(data, 100000);

  // const result = await Promise.all(
  //   chunks.map((chunk) =>
  //     db.insert(collections).values(chunk).onConflictDoNothing()
  //   )
  // );

  for (let i = 0; i < chunks.length; i++) {
    const chunk = chunks[i];
    console.debug(`Inserting chunk ${i}/${chunks.length}`);
    await db.insert(collections).values(chunk).onConflictDoNothing();
  }
};
