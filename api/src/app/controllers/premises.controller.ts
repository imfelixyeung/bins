import { Request, Response } from "express";

import { getManyByPostcode, getOne } from "../services/premises.service";

export const getById = async (req: Request, res: Response) => {
  const { id } = req.params;

  const result = await getOne(id);

  if (!result) {
    return res.status(404).send({
      message: `No premises found with id ${id}`,
    });
  }

  return res.status(200).send(result);
};

export const getByPostcode = async (req: Request, res: Response) => {
  const { postcode } = req.query;

  if (!postcode) {
    return res.status(400).send({
      message: "No postcode provided",
    });
  }

  const result = await getManyByPostcode(postcode?.toString());

  if (!result) {
    return res.status(404).send({
      message: `No premises found with postcode ${postcode}`,
    });
  }

  return res.status(200).send(result);
};
