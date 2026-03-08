const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const https = require("https");

const REPO = "ezani92/bitart-generator";
const BIN_DIR = path.join(__dirname, "bin");
const BIN_PATH = path.join(BIN_DIR, "bitart");

function getPlatformKey() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "darwin" && arch === "arm64") return "macos-arm64";
  if (platform === "darwin" && arch === "x64") return "macos-x86_64";
  if (platform === "linux" && arch === "x64") return "linux-x86_64";

  throw new Error(
    `Unsupported platform: ${platform}-${arch}. ` +
      `Install from source: cargo install bitart-generator`
  );
}

function getVersion() {
  const pkg = require("./package.json");
  return `v${pkg.version}`;
}

function download(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (res) => {
        if (res.statusCode === 302 || res.statusCode === 301) {
          return download(res.headers.location).then(resolve).catch(reject);
        }
        if (res.statusCode !== 200) {
          return reject(new Error(`Download failed: HTTP ${res.statusCode}`));
        }
        const chunks = [];
        res.on("data", (chunk) => chunks.push(chunk));
        res.on("end", () => resolve(Buffer.concat(chunks)));
        res.on("error", reject);
      })
      .on("error", reject);
  });
}

async function install() {
  const platformKey = getPlatformKey();
  const version = getVersion();
  const tarName = `bitart-${version}-${platformKey}.tar.gz`;
  const url = `https://github.com/${REPO}/releases/download/${version}/${tarName}`;

  console.log(`Downloading bitart ${version} for ${platformKey}...`);

  const data = await download(url);

  // Write tarball to temp file
  const tmpTar = path.join(__dirname, tarName);
  fs.writeFileSync(tmpTar, data);

  // Ensure bin directory exists
  if (!fs.existsSync(BIN_DIR)) {
    fs.mkdirSync(BIN_DIR, { recursive: true });
  }

  // Extract
  execSync(`tar xzf "${tmpTar}" -C "${BIN_DIR}"`, { stdio: "inherit" });

  // Clean up tarball
  fs.unlinkSync(tmpTar);

  // Make executable
  fs.chmodSync(BIN_PATH, 0o755);

  console.log(`bitart ${version} installed successfully!`);
}

install().catch((err) => {
  console.error(`Failed to install bitart: ${err.message}`);
  console.error("You can install from source instead: cargo install bitart-generator");
  process.exit(1);
});
