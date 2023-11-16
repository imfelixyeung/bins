// premises.route.ts

import { Router } from "express";

import { getById, getByPostcode } from "../controllers/premises.controller";

export const premisesRouter: Router = Router();

premisesRouter.get("/:id", getById);
premisesRouter.get("/", getByPostcode);
