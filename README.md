# Keplerian-sim Demo

This is a demo of my Keplerian orbital simulator Rust library: https://crates.io/crates/keplerian_sim

It includes a simple web interface to visualize the simulation and interact with it.

## Building

To transpile the TypeScript code to JavaScript, you'll need a TS to JS transpiler. I personally prefer Bun:

```bash
bun build --production --minify --outfile=assets/watchdog.js assets/watchdog.ts
```

`trunk` (https://trunkrs.dev/) is used to build the application. To serve it:
```bash
trunk serve
```

Build for production:
```bash
trunk build --release
```