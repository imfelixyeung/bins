export const getFullAddress = (data: {
  addressRoom?: string | null | undefined;
  addressNumber?: string | null | undefined;
  addressStreet?: string | null | undefined;
  addressLocality?: string | null | undefined;
  addressCity?: string | null | undefined;
  addressPostcode?: string | null | undefined;
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

  return fullAddress;
};

export const getSummaryAddress = (data: {
  addressRoom?: string | null | undefined;
  addressNumber?: string | null | undefined;
  addressStreet?: string | null | undefined;
}) => {
  const fullAddress = [data.addressRoom, data.addressNumber, data.addressStreet]
    .filter(Boolean)
    .join(" ");

  return fullAddress;
};
