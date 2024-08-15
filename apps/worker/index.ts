import execa from "execa";
import schedule from "node-schedule";

// every day at 2:00 AM
const cron = "0 2 * * *";

const run = async () => {};

schedule.scheduleJob(cron, run);
