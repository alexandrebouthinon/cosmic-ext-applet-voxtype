// semantic-release configuration, run locally via `just release`.
//
// Both `main` and `develop` tag, write GitHub release notes, update
// CHANGELOG.md and bump Cargo.toml/Cargo.lock. Only `main` additionally
// builds the release binary (after the version bump) and attaches it to
// the GitHub release; `develop` publishes a prerelease with no binary.
import { execSync } from "node:child_process";

const BINARY_NAME = "cosmic-ext-applet-voxtype";
const BINARY_ASSET_NAME = `${BINARY_NAME}-x86_64-linux`;

// Read the branch locally rather than from a CI env var: `just release`
// always runs on the developer's machine, never in CI.
const branch = execSync("git rev-parse --abbrev-ref HEAD").toString().trim();
const isMain = branch === "main";

export default {
  branches: [
    "main",
    { name: "develop", channel: "develop", prerelease: "develop" },
  ],
  tagFormat: "v${version}",
  plugins: [
    "@semantic-release/commit-analyzer",
    "@semantic-release/release-notes-generator",
    "@semantic-release/changelog",
    {
      // Only the `prepare` step of semantic-release-cargo is used here: it
      // bumps Cargo.toml/Cargo.lock without requiring a
      // CARGO_REGISTRY_TOKEN and never calls `cargo publish` — this crate
      // is not published to crates.io.
      prepare: async (pluginConfig, context) => {
        const { prepare } = await import(
          "@semantic-release-cargo/semantic-release-cargo"
        );
        context.logger.log(
          `Setting Cargo.toml version to ${context.nextRelease.version}`,
        );
        await prepare(pluginConfig, context);
      },
    },
    // Build only on `main`, after the version bump above so the binary
    // embeds the released version. The gh token is dropped so no
    // dependency build script can read it.
    ...(isMain
      ? [
          [
            "@semantic-release/exec",
            {
              prepareCmd:
                "env -u GITHUB_TOKEN -u GH_TOKEN cargo build --release",
            },
          ],
        ]
      : []),
    [
      "@semantic-release/git",
      {
        assets: ["CHANGELOG.md", "Cargo.toml", "Cargo.lock"],
        message: "chore(release): ${nextRelease.version} [skip ci]",
      },
    ],
    [
      "@semantic-release/github",
      {
        assets: isMain
          ? [
              {
                path: `target/release/${BINARY_NAME}`,
                label: BINARY_ASSET_NAME,
              },
            ]
          : [],
      },
    ],
  ],
};
