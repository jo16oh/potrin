import { DockerComposeEnvironment, Wait } from "testcontainers";
import { exec } from "node:child_process";
import util from "node:util";

const targetLog =
  'LOG:  logical replication apply worker for subscription "postgres_1" has started';
const commands = `
pnpm run drizzle push --dialect='postgresql' --schema='./db/schema.ts' --url=postgresql://postgres:proxy_password@localhost:65433 &&
export PGPASSWORD='proxy_password' &&
psql -h localhost -p 65433 -U postgres -a -f db/electrify.sql &&
pnpm exec electric-sql generate --proxy postgresql://postgres:proxy_password@localhost:65433/postgres -s http://localhost:5134
`;

export async function setup() {
  console.log("🐳 setting up testcontainers...");
  await new DockerComposeEnvironment("./docker", "compose.test.yml")
    .withNoRecreate()
    .withWaitStrategy("postgres-1", Wait.forLogMessage(targetLog))
    .up();
  const { stdout } = await util.promisify(exec)(commands);
  console.log(stdout);
}
