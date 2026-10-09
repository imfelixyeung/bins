import { getPresentableFullAddress } from "@/functions/format-address";

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
  const fullAddress = getPresentableFullAddress(data);

  return <div className="whitespace-pre-line">{fullAddress}</div>;
};

export default Address;
