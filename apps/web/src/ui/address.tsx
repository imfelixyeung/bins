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
  const fullAddress = [
    data.addressRoom,
    data.addressNumber,
    data.addressStreet,
    data.addressLocality,
    data.addressCity,
    data.addressPostcode,
  ]
    .filter(Boolean)
    .join("\n");

  return <div className="whitespace-pre-line">{fullAddress}</div>;
};

export default Address;
