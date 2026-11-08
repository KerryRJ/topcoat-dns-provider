# Project conventions

- Keep the crate reusable across applications. Do not couple DNS operations to a particular application, database, authentication system, or theme.
- DNS provider I/O must use asynchronous APIs. Preserve unrelated DNS records when removing a value.
- Never log credentials or return saved secrets to the UI. Applications own authorization and secure credential storage.
- Use Topcoat UI components and shared icons. Keep application-specific adapters outside installed `src/components`.
- Support host styling and light and dark themes. Left-align labels beside bounded fields; allow fields to shrink without overflow. Buttons must not stretch and must show progress while operations are running.
- Verify UI changes in a browser before claiming visual or interaction correctness. Report compilation, tests, browser checks, and live provider checks separately.
- Keep `components.toml` in sync with components installed by `topcoat ui`; do not invent installation hashes.
- Use subagents for suitable independent tasks when available, and review their findings and changes before completing the work.
