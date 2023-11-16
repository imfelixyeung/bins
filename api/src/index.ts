import { app, start } from "./app";
import { populateCollections, populatePremises } from "./populate-db";

start(app);

app.get("/populate", async (req, res) => {
  await populatePremises();
  await populateCollections();
  res.send("done");
});
