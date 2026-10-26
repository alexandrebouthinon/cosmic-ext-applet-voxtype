// semantic-release configuration, run by the release workflow on every push
// to `main` or `develop`.
//
// Both `main` and `develop` tag, write GitHub release notes, update
// CHANGELOG.md and bump Cargo.toml/Cargo.lock. Only `main` additionally
// builds the binary and the AppImage (after the version bump) and attaches
// them to the GitHub release; `develop` publishes a prerelease with no assets.
const BINARY_NAME = "cosmic-ext-applet-voxtype";
const isMain = process.env.GITHUB_REF_NAME === "main";

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
                "env -u GITHUB_TOKEN -u GH_TOKEN just appimage",
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
                name: `${BINARY_NAME}-x86_64-linux`,
              },
              { path: `target/${BINARY_NAME}-x86_64.AppImage` },
            ]
          : [],
      },
    ],
  ],
};
