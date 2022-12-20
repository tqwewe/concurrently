import { copyAssets } from "./copyAssets.task.ts";

Deno.test(`copy assets`, async () => {
  await copyAssets();
});
