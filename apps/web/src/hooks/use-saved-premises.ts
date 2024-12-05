"use client";
import { useLocalStorage } from "@uidotdev/usehooks";
import { useMemo } from "react";
import { z } from "zod";

export const premisesSchema = z.object({
  id: z.number(),
  addressRoom: z.string().nullable(),
  addressNumber: z.string().nullable(),
  addressStreet: z.string().nullable(),
  addressLocality: z.string().nullable(),
  addressCity: z.string().nullable(),
  addressPostcode: z.string().nullable(),
});

export type Premises = z.infer<typeof premisesSchema>;

export const localStorageSchema = z.object({
  premises: z.array(premisesSchema),
});

export type LocalStorage = z.infer<typeof localStorageSchema>;

const defaultLocalStorage: LocalStorage = {
  premises: [],
};

export const useSavedPremises = () => {
  const MAX_SAVED_PREMISES = 10;
  const [localPremises, saveLocalPremises] = useLocalStorage<LocalStorage>(
    "bins__saved_premises",
    defaultLocalStorage
  );

  const premises: Premises[] = useMemo(() => {
    try {
      const parsed = localStorageSchema.safeParse(localPremises);

      if (parsed.success) {
        return parsed.data.premises;
      }

      return [];
    } catch (error) {
      console.warn("Unable to parse saved premises, using empty array", error, {
        localPremises,
      });
      return [];
    }
  }, [localPremises]);

  const add = (_newPremises: Premises) => {
    const parsed = premisesSchema.safeParse(_newPremises);

    if (!parsed.success) {
      console.warn("Unable to add premises to saved premises", parsed.error);
      return;
    }

    const newPremises = parsed.data;

    const newPremisesArray = [
      newPremises,
      ...premises.filter((p) => p.id !== newPremises.id),
    ].slice(0, MAX_SAVED_PREMISES);

    saveLocalPremises({
      premises: newPremisesArray,
    });
  };

  const remove = (id: Premises["id"]) => {
    const newPremisesArray = premises.filter((p) => p.id !== id);

    saveLocalPremises({
      premises: newPremisesArray,
    });
  };

  const removeAll = () => {
    saveLocalPremises({
      premises: [],
    });
  };

  return {
    premises,
    add,
    remove,
    removeAll,
  };
};
