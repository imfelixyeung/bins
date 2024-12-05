import { capitalCase } from "change-case";

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

export const getPresentableFullAddress = (data: {
  addressRoom?: string | null | undefined;
  addressNumber?: string | null | undefined;
  addressStreet?: string | null | undefined;
  addressLocality?: string | null | undefined;
  addressCity?: string | null | undefined;
  addressPostcode?: string | null | undefined;
}) => {
  const fullAddress = [
    data.addressRoom ? capitalCase(data.addressRoom) : data.addressRoom,
    data.addressNumber ? capitalCase(data.addressNumber) : data.addressNumber,
    data.addressStreet ? capitalCase(data.addressStreet) : data.addressStreet,
    data.addressLocality
      ? capitalCase(data.addressLocality)
      : data.addressLocality,
    data.addressCity ? capitalCase(data.addressCity) : data.addressCity,
    data.addressPostcode?.toUpperCase(),
  ]
    .filter(Boolean)
    .join("\n");

  return fullAddress;
};

export const getOneLineFullAddress = (data: {
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
    .join(", ");

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
