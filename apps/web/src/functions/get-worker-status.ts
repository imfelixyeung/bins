import ky from "ky";

export const getWorkerStatus = async () => {
  const response = await ky.get("http://worker:3000/status");
  return response.json();
};
