export const getWorkerStatus = async () => {
  const response = await fetch("http://worker:3000/status");

  if (!response.ok) {
    throw new Error("Failed to get worker status");
  }

  if (response.status !== 200) {
    throw new Error("Failed to get worker status");
  }

  return response.json();
};
