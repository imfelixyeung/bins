"use client";

import React, { useEffect } from "react";
import { type Premises, useSavedPremises } from "@/hooks/use-saved-premises";

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
