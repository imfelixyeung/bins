export const triggerWorkerUpdate = async () => {
  const response = await fetch("http://worker:3000/update", { method: "POST" });
  return response.json();
};
