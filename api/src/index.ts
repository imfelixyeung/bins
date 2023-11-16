import express from "express";
import cors from "cors";

const app = express();

app.use(cors());

app.get("/", (req, res) => {
  res.send("Hello world!");
});

const PORT = 80;

app.listen(PORT, () => {
  console.log(`Server listening on port ${PORT}`);
});
