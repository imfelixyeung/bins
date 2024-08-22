"use client";

import { Premises, useSavedPremises } from "@/hooks/use-saved-premises";
import React, { useEffect } from "react";

const AddToRecents = ({ premises }: { premises: Premises }) => {
  const [added, setAdded] = React.useState(false);
  const { add } = useSavedPremises();

  useEffect(() => {
    if (added) return;
    add(premises);
    setAdded(true);
  }, [premises, add, added]);

  return null;
};

export default AddToRecents;
