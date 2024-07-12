import { exec } from "node:child_process";
import util from "node:util";

export async function setup() {
  console.log("🐳 setting up containers...");
  const { stdout, stderr } = await util.promisify(exec)("./db/test-setup.sh");
  if (stdout) console.log(stdout);
  if (stderr) console.log(stderr);
}
