export const getWorkerStatus = async () => {
  const response = await fetch("http://worker:3000/status");
  return response.json();
};
