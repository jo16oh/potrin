import { DockerComposeEnvironment, Wait } from "testcontainers";
import { exec } from "node:child_process";
import util from "node:util";

const targetLog =
  'LOG:  logical replication apply worker for subscription "postgres_1" has started';
const commands = `
pnpm pg-migrations apply --directory db/migrations --database postgresql://postgres:proxy_password@localhost:65433/postgres &&
pnpm exec electric-sql generate --proxy postgresql://postgres:proxy_password@localhost:65433/postgres -s http://localhost:5134
`;

export async function setup() {
  console.log("🐳 setting up testcontainers...");
  await new DockerComposeEnvironment("./docker", "compose.test.yml")
    .withWaitStrategy("postgres-1", Wait.forLogMessage(targetLog))
    .up();
  const { stdout } = await util.promisify(exec)(commands);
  console.log(stdout);
}
