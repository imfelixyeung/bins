export const triggerWorkerUpdate = async () => {
  const response = await fetch("http://worker:3000/update", { method: "POST" });

  if (!response.ok) {
    throw new Error("Failed to trigger worker update");
  }

  if (response.status !== 200) {
    throw new Error("Failed to trigger worker update");
  }

  return response.json();
};
