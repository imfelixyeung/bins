import ky from "ky";

export const triggerWorkerUpdate = async () => {
  const response = await ky.post("http://worker:3000/update");
  return response.json();
};
