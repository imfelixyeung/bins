"use client";

import { useEffect } from "react";
import { type Premises, useSavedPremises } from "@/hooks/use-saved-premises";

const AddToRecents = ({ premises }: { premises: Premises }) => {
  const { add } = useSavedPremises();

  useEffect(() => {
    add(premises);
  }, [premises, add]);

  return null;
};

export default AddToRecents;
