#!/usr/bin/env node

const { execFileSync } = require("child_process");
const path = require("path");

const binary = path.join(__dirname, "bitart");

try {
  execFileSync(binary, process.argv.slice(2), { stdio: "inherit" });
} catch (e) {
  if (e.status !== null) {
    process.exit(e.status);
  }
  console.error("Failed to run bitart:", e.message);
  process.exit(1);
}
