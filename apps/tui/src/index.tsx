#!/usr/bin/env bun
/**
 * c0mpute-tui — interactive terminal dashboard.
 *
 * Launched by `c0mpute tui ...`. Long-term this becomes the
 * worker / job / module dashboard with live data; today it's a
 * react-blessed scaffold so the wiring is real even if the views
 * are placeholder.
 *
 * See dips/0008-ui-strategy.md.
 */

import blessed from "blessed";
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore — react-blessed has no types but works fine
import { render } from "react-blessed";
import * as React from "react";

const screen = blessed.screen({
  smartCSR: true,
  title: "c0mpute · tui",
  fullUnicode: true,
});

screen.key(["escape", "q", "C-c"], () => process.exit(0));

function App() {
  return (
    <box
      top="center"
      left="center"
      width="80%"
      height="80%"
      border={{ type: "line" }}
      style={{ border: { fg: "green" }, fg: "white", bg: "black" }}
      label=" c0mpute "
    >
      <text top={1} left={2} content="c0mpute · interactive dashboard" style={{ fg: "green" }} />
      <text top={3} left={2} content="// scaffold — react-blessed wiring works; views land in Phase 2" style={{ fg: "grey" }} />

      <text top={5} left={2} content="[ workers ]" style={{ fg: "green" }} />
      <text top={6} left={4} content="(none registered yet)" style={{ fg: "grey" }} />

      <text top={8} left={2} content="[ jobs ]" style={{ fg: "green" }} />
      <text top={9} left={4} content="(none queued)" style={{ fg: "grey" }} />

      <text top={11} left={2} content="[ modules ]" style={{ fg: "green" }} />
      <text top={12} left={4} content="transcode  · coinpay  · infernet" style={{ fg: "white" }} />

      <text bottom={1} left={2} content="q / esc / ctrl-c — quit" style={{ fg: "grey" }} />
    </box>
  );
}

render(<App />, screen);
