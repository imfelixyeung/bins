import { getFullAddress } from "@/functions/format-address";
import React from "react";

const Address = ({
  data,
}: {
  data: {
    addressRoom: string | null;
    addressNumber: string | null;
    addressStreet: string | null;
    addressLocality: string | null;
    addressCity: string | null;
    addressPostcode: string | null;
  };
}) => {
  const fullAddress = getFullAddress(data);

  return <div className="whitespace-pre-line">{fullAddress}</div>;
};

export default Address;
