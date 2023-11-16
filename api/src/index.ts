import { app, start } from "./app";
import { populateCollections } from "./populate-db";

start(app);

app.get("/test", async (req, res) => {
  await populateCollections();
  res.send("done");
});
